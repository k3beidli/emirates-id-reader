//! Public connection lifecycle, serialized reads, and library entry points.
use crate::{CardGeneration, EmiratesIdData, Error, ErrorKind, ReadOptions};
use crate::{protocol::Reader, transport::Connection};

/// A live native PC/SC connection to one inserted card.
///
/// Reads block the calling thread. Reuse the session for presence checks, and
/// reconnect after removal or reset. Dropping it releases native resources.
/// Concurrent reads on the same session are serialized for the entire read.
pub struct CardSession {
    connection: Connection,
}

impl CardSession {
    /// Lists installed PC/SC readers. An empty list means no reader is installed.
    ///
    /// # Errors
    /// Returns a PC/SC error if the system service cannot enumerate readers.
    pub fn reader_names() -> Result<Vec<String>, Error> {
        Connection::reader_names()
    }
    /// Connects to the exact reader name returned by [`Self::reader_names`].
    ///
    /// # Errors
    /// Returns `InvalidArgument` for an empty name or embedded NUL, or a native connection error.
    pub fn connect(reader_name: &str) -> Result<Self, Error> {
        if reader_name.is_empty() || reader_name.contains('\0') {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Reader name must be nonempty and contain no NUL",
            ));
        }
        Ok(Self {
            connection: Connection::connect(reader_name)?,
        })
    }
    /// Connects to the first accessible reader containing a card.
    /// Non-presence failures are preserved if no connection succeeds.
    ///
    /// # Errors
    /// Returns `NoReader`, `NoCard`, or the first non-absence connection failure.
    pub fn connect_first() -> Result<Self, Error> {
        Ok(Self {
            connection: Connection::connect_first()?,
        })
    }
    /// Returns the reader name, without accessing the chip.
    pub fn reader_name(&self) -> &str {
        self.connection.reader_name()
    }
    /// Returns the Answer to Reset captured at connection time.
    pub fn atr(&self) -> &[u8] {
        self.connection.atr()
    }
    /// Returns uppercase, space-separated ATR bytes.
    pub fn atr_hex(&self) -> String {
        self.connection.atr_hex()
    }
    /// Classifies the ATR. This does not authenticate the card.
    pub fn card_generation(&self) -> CardGeneration {
        self.connection.card_generation()
    }
    /// Checks card presence without reading identity data.
    ///
    /// # Errors
    /// Returns a PC/SC or synchronization error when presence cannot be determined. Removal returns `Ok(false)`.
    pub fn is_present(&self) -> Result<bool, Error> {
        self.connection.is_present()
    }
    /// Disconnects explicitly, allowing the caller to observe cleanup errors.
    /// This consumes the session and may block in the native driver.
    ///
    /// # Errors
    /// Returns a PC/SC error if disconnection fails. Dropping a session instead
    /// performs best-effort cleanup without reporting errors.
    pub fn close(self) -> Result<(), Error> {
        self.connection.close()
    }
    /// Reads only identifiers and core identity.
    ///
    /// # Errors
    /// See [`Self::read_with_options`] for read failures.
    pub fn read_identity(&self) -> Result<EmiratesIdData, Error> {
        self.read_with_options(ReadOptions::identity_only())
    }
    /// Reads all supported public groups into an owned snapshot.
    ///
    /// # Errors
    /// See [`Self::read_with_options`] for read failures.
    pub fn read_all(&self) -> Result<EmiratesIdData, Error> {
        self.read_with_options(ReadOptions::all())
    }
    /// Reads identifiers and core identity plus the requested optional groups.
    /// Transport and malformed-data errors fail the read; inaccessible optional
    /// groups are represented by [`crate::ReadStatus`].
    ///
    /// # Errors
    /// Returns an error if required data cannot be read, any returned data is malformed, or transport/transaction cleanup fails. No partial snapshot is returned.
    pub fn read_with_options(&self, options: ReadOptions) -> Result<EmiratesIdData, Error> {
        self.connection.with_transaction(|transaction| {
            Reader {
                exchange: |command: &[u8]| crate::transport::transmit(transaction, command),
            }
            .read(self.reader_name(), self.card_generation(), options)
        })
    }
}

impl std::fmt::Debug for CardSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardSession")
            .field("card_generation", &self.card_generation())
            .finish_non_exhaustive()
    }
}
