//! Structured library errors.
#[cfg(feature = "serde")]
use serde::Serialize;
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
/// Stable categories callers can use to decide whether to wait, reconnect, or reject data.
#[non_exhaustive]
pub enum ErrorKind {
    /// A library argument is invalid.
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

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// Error returned by the library.
#[non_exhaustive]
pub struct Error {
    /// Machine-readable error category.
    pub kind: ErrorKind,
    /// Human-readable diagnostic with no cardholder field values.
    pub message: String,
    /// ISO 7816 status word when the failure came from the card itself.
    pub status_word: Option<u16>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pcsc_code: Option<u32>,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            status_word: None,
            pcsc_code: None,
        }
    }

    pub(crate) fn apdu(status_word: u16) -> Self {
        Self {
            kind: ErrorKind::Protocol,
            message: format!("Card APDU failed with status word {status_word:04X}"),
            status_word: Some(status_word),
            pcsc_code: None,
        }
    }

    /// Returns the original native PC/SC error code, when applicable.
    pub fn pcsc_code(&self) -> Option<u32> {
        self.pcsc_code
    }

    pub(crate) fn pcsc(operation: &str, error: pcsc::Error) -> Self {
        let kind = match error {
            pcsc::Error::NoReadersAvailable => ErrorKind::NoReader,
            pcsc::Error::NoSmartcard => ErrorKind::NoCard,
            pcsc::Error::RemovedCard | pcsc::Error::ResetCard | pcsc::Error::ReaderUnavailable => {
                ErrorKind::CardRemoved
            }
            _ => ErrorKind::Pcsc,
        };
        Self {
            kind,
            message: format!("{operation} failed with PC/SC error 0x{:08X}", error as u32),
            status_word: None,
            pcsc_code: Some(error as u32),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}
impl std::error::Error for Error {}
