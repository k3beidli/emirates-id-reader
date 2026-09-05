//! Windows PC/SC ownership and FFI. Card is dropped before its context.
use crate::apdu::exchange_apdu;
use crate::{CardGeneration, Error, ErrorKind};
use std::ffi::{OsString, c_void};
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
const SCARD_E_NO_READERS_AVAILABLE: Long = 0x8010_002E_u32 as i32;

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

pub(crate) struct Transaction<'card>(&'card Card);

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

/// A live connection to one inserted card. Keep it alive to monitor removal
/// without repeatedly reading identity data from the chip.
pub(crate) struct Connection {
    card: Card,
    _context: Context,
    protocol: Dword,
    atr: Vec<u8>,
    reader_name: String,
}

impl Connection {
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
        let mut connection_error = None;
        for reader in readers {
            match connect_reader(&context, &reader) {
                Ok((card, protocol, atr)) => {
                    return Ok(Self {
                        _context: context,
                        card,
                        protocol,
                        atr,
                        reader_name: reader.to_string_lossy().into_owned(),
                    });
                }
                Err(error) if error.kind == ErrorKind::NoCard => {}
                Err(error) => {
                    connection_error.get_or_insert(error);
                }
            }
        }
        Err(connection_error
            .unwrap_or_else(|| Error::new(ErrorKind::NoCard, "No inserted smart card was found")))
    }

    pub(crate) fn begin_transaction(&self) -> Result<Transaction<'_>, Error> {
        Transaction::begin(&self.card)
    }
    pub(crate) fn transmit(&self, command: &[u8]) -> Result<Vec<u8>, Error> {
        transmit(&self.card, self.protocol, command)
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
    if result == SCARD_E_NO_READERS_AVAILABLE {
        return Ok(Vec::new());
    }
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
    let atr = atr
        .get(..atr_len as usize)
        .ok_or_else(|| Error::new(ErrorKind::Protocol, "PC/SC returned an invalid ATR length"))?;
    Ok((protocol, atr.to_vec()))
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
    if response_len as usize > response.len() {
        return Err(Error::new(
            ErrorKind::Protocol,
            "PC/SC returned an invalid response length",
        ));
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
