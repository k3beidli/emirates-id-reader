//! Structured SDK errors.
use serde::Serialize;
use std::fmt;
const SCARD_E_NO_SMARTCARD: i32 = 0x8010_000C_u32 as i32;
const SCARD_E_READER_UNAVAILABLE: i32 = 0x8010_0017_u32 as i32;
const SCARD_E_NO_READERS_AVAILABLE: i32 = 0x8010_002E_u32 as i32;
const SCARD_W_RESET_CARD: i32 = 0x8010_0068_u32 as i32;
const SCARD_W_REMOVED_CARD: i32 = 0x8010_0069_u32 as i32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
/// Stable categories callers can use to decide whether to wait, reconnect, or reject data.
pub enum ErrorKind {
    /// An SDK argument is invalid.
    InvalidArgument,
    /// native PC/SC reports no installed reader.
    NoReader,
    /// Readers exist, but none currently contains a card.
    NoCard,
    /// A connected card or reader disappeared or reset.
    CardRemoved,
    /// A native PC/SC operation failed.
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
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status_word: None,
        }
    }

    pub(crate) fn apdu(status_word: u16) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            message: format!("Card APDU failed with status word {status_word:04X}"),
            status_word: Some(status_word),
        }
    }

    pub(crate) fn pcsc(operation: &str, code: i32) -> Self {
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
