//! A Rust library for reading data from Emirates ID chips.
//!
//! The crate talks to contact cards through Windows PC/SC. Cardholder data is
//! returned in memory and is never persisted by the SDK.
//!
//! # Quick start
//!
//! ```no_run
//! use emirates_id_reader::{CardSession, ReadOptions};
//!
//! # fn main() -> Result<(), emirates_id_reader::Error> {
//! let session = CardSession::connect_first()?;
//! let card = session.read_with_options(ReadOptions::identity_only())?;
//! println!("{}", card.id_number);
//! # Ok(())
//! # }
//! ```
//!
//! Use [`CardSession::read`] when photographs and all other supported public
//! groups are required. Inspect [`EmiratesIdData::read_status`] to distinguish
//! a blank field from an optional group that was protected, unavailable, or
//! deliberately skipped.

#![warn(missing_docs)]

mod apdu;
mod data;
mod decode;

use apdu::exchange_apdu;
pub use data::*;
use decode::{decode_modifiable, decode_non_modifiable, field, required_ascii_digits};

#[cfg(test)]
mod tests;

use serde::Serialize;
use std::ffi::{OsString, c_void};
use std::fmt;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::ptr::{null, null_mut};

type ScardContext = usize;
type ScardHandle = usize;
type Long = i32;
type Dword = u32;

const SCARD_SCOPE_USER: Dword = 0;
const SCARD_SHARE_SHARED: Dword = 2;
const SCARD_PROTOCOL_T0: Dword = 1;
const SCARD_PROTOCOL_T1: Dword = 2;
const SCARD_S_SUCCESS: Long = 0;
const SCARD_E_NO_SMARTCARD: Long = 0x8010_000C_u32 as i32;
const SCARD_E_READER_UNAVAILABLE: Long = 0x8010_0017_u32 as i32;
const SCARD_E_NO_READERS_AVAILABLE: Long = 0x8010_002E_u32 as i32;
const SCARD_W_RESET_CARD: Long = 0x8010_0068_u32 as i32;
const SCARD_W_REMOVED_CARD: Long = 0x8010_0069_u32 as i32;

const PUBLIC_DATA_DIRECTORY: u16 = 0x0200;
const IDENTITY_FILE: u16 = 0x0201;
const PHOTO_FILE: u16 = 0x0202;
const NON_MODIFIABLE_FILE: u16 = 0x0203;
const MODIFIABLE_FILE: u16 = 0x0205;
const HOLDER_SIGNATURE_FILE: u16 = 0x0207;

#[derive(Clone, Copy)]
enum PublicFileLayout {
    ApplicationRoot,
    Directory,
}

#[repr(C)]
struct ScardIoRequest {
    protocol: Dword,
    pci_length: Dword,
}

#[link(name = "winscard")]
unsafe extern "system" {
    fn SCardEstablishContext(
        dw_scope: Dword,
        pv_reserved_1: *const c_void,
        pv_reserved_2: *const c_void,
        ph_context: *mut ScardContext,
    ) -> Long;
    fn SCardReleaseContext(h_context: ScardContext) -> Long;
    fn SCardListReadersW(
        h_context: ScardContext,
        msz_groups: *const u16,
        msz_readers: *mut u16,
        pcch_readers: *mut Dword,
    ) -> Long;
    fn SCardConnectW(
        h_context: ScardContext,
        sz_reader: *const u16,
        dw_share_mode: Dword,
        dw_preferred_protocols: Dword,
        ph_card: *mut ScardHandle,
        pdw_active_protocol: *mut Dword,
    ) -> Long;
    fn SCardStatusW(
        h_card: ScardHandle,
        msz_reader_names: *mut u16,
        pcch_reader_len: *mut Dword,
        pdw_state: *mut Dword,
        pdw_protocol: *mut Dword,
        pb_atr: *mut u8,
        pcb_atr_len: *mut Dword,
    ) -> Long;
    fn SCardBeginTransaction(h_card: ScardHandle) -> Long;
    fn SCardEndTransaction(h_card: ScardHandle, dw_disposition: Dword) -> Long;
    fn SCardDisconnect(h_card: ScardHandle, dw_disposition: Dword) -> Long;
    fn SCardTransmit(
        h_card: ScardHandle,
        pio_send_pci: *const ScardIoRequest,
        pb_send_buffer: *const u8,
        cb_send_length: Dword,
        pio_recv_pci: *mut ScardIoRequest,
        pb_recv_buffer: *mut u8,
        pcb_recv_length: *mut Dword,
    ) -> Long;
}

struct Context(ScardContext);
impl Drop for Context {
    fn drop(&mut self) {
        unsafe { SCardReleaseContext(self.0) };
    }
}

struct Card(ScardHandle);
impl Drop for Card {
    fn drop(&mut self) {
        unsafe { SCardDisconnect(self.0, 0) };
    }
}

struct Transaction<'card>(&'card Card);

impl<'card> Transaction<'card> {
    fn begin(card: &'card Card) -> Result<Self, Error> {
        let result = unsafe { SCardBeginTransaction(card.0) };
        if result != SCARD_S_SUCCESS {
            return Err(Error::pcsc("SCardBeginTransaction", result));
        }
        Ok(Self(card))
    }
}

impl Drop for Transaction<'_> {
    fn drop(&mut self) {
        unsafe { SCardEndTransaction(self.0.0, 0) };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Stable categories callers can use to decide whether to wait, reconnect, or reject data.
pub enum ErrorKind {
    /// Windows PC/SC reports no installed reader.
    NoReader,
    /// Readers exist, but none currently contains a card.
    NoCard,
    /// A connected card or reader disappeared or reset.
    CardRemoved,
    /// A Windows PC/SC operation failed.
    Pcsc,
    /// The card rejected an APDU or returned an invalid protocol response.
    Protocol,
    /// A card file was present but malformed or could not be decoded safely.
    InvalidData,
}

#[derive(Clone, Debug, Serialize)]
/// Error returned by the SDK.
pub struct Error {
    /// Machine-readable error category.
    pub kind: ErrorKind,
    /// Human-readable diagnostic with no cardholder field values.
    pub message: String,
    /// ISO 7816 status word when the failure came from the card itself.
    pub status_word: Option<u16>,
}

impl Error {
    fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status_word: None,
        }
    }

    fn apdu(status_word: u16) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            message: format!("Card APDU failed with status word {status_word:04X}"),
            status_word: Some(status_word),
        }
    }

    fn pcsc(operation: &str, code: Long) -> Self {
        let kind = match code {
            SCARD_E_NO_READERS_AVAILABLE => ErrorKind::NoReader,
            SCARD_E_NO_SMARTCARD => ErrorKind::NoCard,
            SCARD_W_REMOVED_CARD | SCARD_W_RESET_CARD | SCARD_E_READER_UNAVAILABLE => {
                ErrorKind::CardRemoved
            }
            _ => ErrorKind::Pcsc,
        };
        Self::new(
            kind,
            format!("{operation} failed with PC/SC error 0x{:08X}", code as u32),
        )
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for Error {}

/// A live connection to one inserted card. Keep it alive to monitor removal
/// without repeatedly reading identity data from the chip.
pub struct CardSession {
    _context: Context,
    card: Card,
    protocol: Dword,
    atr: Vec<u8>,
    reader_name: String,
}

impl CardSession {
    /// Lists all Windows PC/SC readers visible to the current user.
    pub fn reader_names() -> Result<Vec<String>, Error> {
        let context = establish_context()?;
        Ok(list_readers(&context)?
            .into_iter()
            .map(|reader| reader.to_string_lossy().into_owned())
            .collect())
    }

    /// Connects to a named PC/SC reader containing a card.
    pub fn connect(reader_name: &str) -> Result<Self, Error> {
        let context = establish_context()?;
        let reader = OsString::from(reader_name);
        let (card, protocol, atr) = connect_reader(&context, &reader)?;
        Ok(Self {
            _context: context,
            card,
            protocol,
            atr,
            reader_name: reader_name.to_owned(),
        })
    }

    /// Connects to the first PC/SC reader that currently contains a card.
    pub fn connect_first() -> Result<Self, Error> {
        let context = establish_context()?;
        let readers = list_readers(&context)?;
        if readers.is_empty() {
            return Err(Error::new(
                ErrorKind::NoReader,
                "No PC/SC reader is available",
            ));
        }
        for reader in readers {
            if let Ok((card, protocol, atr)) = connect_reader(&context, &reader) {
                return Ok(Self {
                    _context: context,
                    card,
                    protocol,
                    atr,
                    reader_name: reader.to_string_lossy().into_owned(),
                });
            }
        }
        Err(Error::new(
            ErrorKind::NoCard,
            "No inserted smart card was found",
        ))
    }

    /// Returns the PC/SC reader name.
    pub fn reader_name(&self) -> &str {
        &self.reader_name
    }
    /// Returns the card's Answer to Reset bytes.
    pub fn atr(&self) -> &[u8] {
        &self.atr
    }
    /// Returns the ATR as uppercase, space-separated hexadecimal bytes.
    pub fn atr_hex(&self) -> String {
        self.atr
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Classifies the card using ICP's published V1/V2 ATR list.
    pub fn card_generation(&self) -> CardGeneration {
        CardGeneration::from_atr(&self.atr)
    }

    /// Checks whether this session's card is still present.
    pub fn is_present(&self) -> Result<bool, Error> {
        match card_status(&self.card) {
            Ok(_) => Ok(true),
            Err(error)
                if matches!(
                    error.kind,
                    ErrorKind::NoCard | ErrorKind::CardRemoved | ErrorKind::NoReader
                ) =>
            {
                Ok(false)
            }
            Err(error) => Err(error),
        }
    }

    /// Reads all supported public groups using [`ReadOptions::default`].
    pub fn read(&self) -> Result<EmiratesIdData, Error> {
        self.read_with_options(ReadOptions::default())
    }

    /// Reads public data while allowing callers to skip expensive optional groups.
    pub fn read_with_options(&self, options: ReadOptions) -> Result<EmiratesIdData, Error> {
        let _transaction = Transaction::begin(&self.card)?;
        let layout = self.select_public_base()?;
        let identity = self.read_file(layout, IDENTITY_FILE)?;
        let (photo, photo_status) = self.read_optional_file(layout, PHOTO_FILE, options.photo)?;
        let non_modifiable = self.read_file(layout, NON_MODIFIABLE_FILE)?;
        let (modifiable, modifiable_status) =
            self.read_optional_file(layout, MODIFIABLE_FILE, options.modifiable_data)?;
        let (signature, signature_status) = self.read_optional_file(
            layout,
            HOLDER_SIGNATURE_FILE,
            options.holder_signature_image,
        )?;
        Ok(EmiratesIdData {
            reader_name: self.reader_name.clone(),
            card_generation: self.card_generation(),
            id_number: required_ascii_digits(&identity, 0xE101, "ID number", 15)?,
            card_number: required_ascii_digits(&identity, 0xE102, "card number", 9)?,
            photo_jpeg: photo
                .as_deref()
                .and_then(|data| field(data, 0x6203).ok().flatten())
                .filter(|value| value.starts_with(&[0xFF, 0xD8, 0xFF]))
                .map(Vec::from),
            holder_signature_image: signature
                .as_deref()
                .and_then(|data| field(data, 0x6732).ok().flatten())
                .map(Vec::from),
            non_modifiable: decode_non_modifiable(&non_modifiable)?,
            modifiable: match modifiable {
                Some(data) => decode_modifiable(&data)?,
                None => ModifiableData::default(),
            },
            read_status: ReadStatus {
                identity: DataGroupStatus::Read,
                photo: photo_status,
                non_modifiable: DataGroupStatus::Read,
                modifiable: modifiable_status,
                holder_signature_image: signature_status,
            },
        })
    }

    fn read_optional_file(
        &self,
        layout: PublicFileLayout,
        file_id: u16,
        requested: bool,
    ) -> Result<(Option<Vec<u8>>, DataGroupStatus), Error> {
        if !requested {
            return Ok((None, DataGroupStatus::NotRequested));
        }
        match self.read_file(layout, file_id) {
            Ok(data) => Ok((Some(data), DataGroupStatus::Read)),
            Err(error) if matches!(error.status_word, Some(0x6982) | Some(0x6985)) => {
                Ok((None, DataGroupStatus::Protected))
            }
            Err(error) if matches!(error.status_word, Some(0x6A82) | Some(0x6A83)) => {
                Ok((None, DataGroupStatus::NotAvailable))
            }
            Err(error) => Err(error),
        }
    }

    fn select_public_base(&self) -> Result<PublicFileLayout, Error> {
        select_application(&self.card, self.protocol)?;
        if select_file(&self.card, self.protocol, PUBLIC_DATA_DIRECTORY).is_ok() {
            Ok(PublicFileLayout::Directory)
        } else {
            select_application(&self.card, self.protocol)?;
            Ok(PublicFileLayout::ApplicationRoot)
        }
    }

    fn read_file(&self, _layout: PublicFileLayout, file_id: u16) -> Result<Vec<u8>, Error> {
        // The selected DF remains current while sibling EFs are selected. The
        // layout argument makes that invariant explicit at each call site.
        select_file(&self.card, self.protocol, file_id)?;
        self.read_selected_file()
    }

    fn read_selected_file(&self) -> Result<Vec<u8>, Error> {
        let mut data = read_binary(&self.card, self.protocol, 0, 0xFD)?;
        if data.len() < 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Emirates ID file header is truncated",
            ));
        }
        let total = 4 + u16::from_be_bytes([data[2], data[3]]) as usize;
        if total > 16 * 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unreasonable card file length: {total}"),
            ));
        }
        while data.len() < total {
            let chunk = read_binary(
                &self.card,
                self.protocol,
                data.len(),
                (total - data.len()).min(0xFD),
            )?;
            if chunk.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Empty READ BINARY chunk",
                ));
            }
            data.extend_from_slice(&chunk);
        }
        data.truncate(total);
        Ok(data)
    }
}

fn establish_context() -> Result<Context, Error> {
    let mut raw_context = 0;
    let result =
        unsafe { SCardEstablishContext(SCARD_SCOPE_USER, null(), null(), &mut raw_context) };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardEstablishContext", result));
    }
    Ok(Context(raw_context))
}

fn list_readers(context: &Context) -> Result<Vec<OsString>, Error> {
    let mut reader_chars = 0;
    let result = unsafe { SCardListReadersW(context.0, null(), null_mut(), &mut reader_chars) };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardListReadersW(size)", result));
    }
    let mut reader_buffer = vec![0u16; reader_chars as usize];
    let result = unsafe {
        SCardListReadersW(
            context.0,
            null(),
            reader_buffer.as_mut_ptr(),
            &mut reader_chars,
        )
    };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardListReadersW(data)", result));
    }
    Ok(wide_multi_string(&reader_buffer))
}

fn connect_reader(context: &Context, reader: &OsString) -> Result<(Card, Dword, Vec<u8>), Error> {
    let mut reader_wide: Vec<u16> = reader.encode_wide().collect();
    reader_wide.push(0);
    let mut raw_card = 0;
    let mut protocol = 0;
    let result = unsafe {
        SCardConnectW(
            context.0,
            reader_wide.as_ptr(),
            SCARD_SHARE_SHARED,
            SCARD_PROTOCOL_T0 | SCARD_PROTOCOL_T1,
            &mut raw_card,
            &mut protocol,
        )
    };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardConnectW", result));
    }
    let card = Card(raw_card);
    let atr = card_status(&card)?.1;
    Ok((card, protocol, atr))
}

fn wide_multi_string(buffer: &[u16]) -> Vec<OsString> {
    buffer
        .split(|character| *character == 0)
        .take_while(|part| !part.is_empty())
        .map(OsString::from_wide)
        .collect()
}

fn card_status(card: &Card) -> Result<(Dword, Vec<u8>), Error> {
    let mut atr = [0u8; 64];
    let mut atr_len = atr.len() as Dword;
    let mut state = 0;
    let mut protocol = 0;
    let mut reader_chars = 0;
    let result = unsafe {
        SCardStatusW(
            card.0,
            null_mut(),
            &mut reader_chars,
            &mut state,
            &mut protocol,
            atr.as_mut_ptr(),
            &mut atr_len,
        )
    };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardStatusW", result));
    }
    Ok((protocol, atr[..atr_len as usize].to_vec()))
}

fn transmit_once(card: &Card, protocol: Dword, request: &[u8]) -> Result<(Vec<u8>, u16), Error> {
    let send_pci = ScardIoRequest {
        protocol,
        pci_length: std::mem::size_of::<ScardIoRequest>() as Dword,
    };
    let mut response = vec![0u8; 65_536];
    let mut response_len = response.len() as Dword;
    let result = unsafe {
        SCardTransmit(
            card.0,
            &send_pci,
            request.as_ptr(),
            request.len() as Dword,
            null_mut(),
            response.as_mut_ptr(),
            &mut response_len,
        )
    };
    if result != SCARD_S_SUCCESS {
        return Err(Error::pcsc("SCardTransmit", result));
    }
    response.truncate(response_len as usize);
    if response.len() < 2 {
        return Err(Error::new(
            ErrorKind::Protocol,
            "Card response has no status word",
        ));
    }
    let status_at = response.len() - 2;
    let status = u16::from_be_bytes([response[status_at], response[status_at + 1]]);
    response.truncate(status_at);
    Ok((response, status))
}

/// Exchange one ISO 7816 command, including the response continuation used by
/// T=0 cards (`61xx`) and a single wrong-length correction (`6Cxx`).
fn transmit(card: &Card, protocol: Dword, request: &[u8]) -> Result<Vec<u8>, Error> {
    exchange_apdu(request, |command| transmit_once(card, protocol, command))
}

fn select_application(card: &Card, protocol: Dword) -> Result<(), Error> {
    const AID: [u8; 12] = [
        0xA0, 0x00, 0x00, 0x02, 0x43, 0x00, 0x13, 0x00, 0x00, 0x00, 0x01, 0x01,
    ];
    let mut command = vec![0x00, 0xA4, 0x04, 0x00, AID.len() as u8];
    command.extend_from_slice(&AID);
    command.push(0x00);
    transmit(card, protocol, &command)?;
    Ok(())
}

fn select_file(card: &Card, protocol: Dword, file_id: u16) -> Result<(), Error> {
    let [high, low] = file_id.to_be_bytes();
    transmit(
        card,
        protocol,
        &[0x00, 0xA4, 0x00, 0x00, 0x02, high, low, 0x00],
    )?;
    Ok(())
}

fn read_binary(
    card: &Card,
    protocol: Dword,
    offset: usize,
    length: usize,
) -> Result<Vec<u8>, Error> {
    if offset > 0x7FFF || length == 0 || length > 0xFD {
        return Err(Error::new(
            ErrorKind::Protocol,
            "Invalid READ BINARY offset or length",
        ));
    }
    transmit(
        card,
        protocol,
        &[
            0x00,
            0xB0,
            ((offset >> 8) & 0x7F) as u8,
            (offset & 0xFF) as u8,
            length as u8,
        ],
    )
}
