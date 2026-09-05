//! Public connection lifecycle, serialized reads, and SDK entry points.
use crate::{CardGeneration, EmiratesIdData, Error, ErrorKind, ReadOptions};
use crate::{protocol::Reader, transport::Connection};
use std::sync::Mutex;

/// A live Windows PC/SC connection to one inserted card.
///
/// Reads block the calling thread. Reuse the session for presence checks, and
/// reconnect after removal or reset. Dropping it releases native resources.
/// Concurrent reads on the same session are serialized for the entire read.
pub struct CardSession {
    connection: Connection,
    read_lock: Mutex<()>,
}

impl CardSession {
    /// Lists installed PC/SC readers. An empty list means no reader is installed.
    pub fn reader_names() -> Result<Vec<String>, Error> {
        Connection::reader_names()
    }
    /// Connects to the exact reader name returned by [`Self::reader_names`].
    pub fn connect(reader_name: &str) -> Result<Self, Error> {
        if reader_name.is_empty() || reader_name.contains('\0') {
            return Err(Error::new(
                ErrorKind::InvalidArgument,
                "Reader name must be nonempty and contain no NUL",
            ));
        }
        Ok(Self {
            connection: Connection::connect(reader_name)?,
            read_lock: Mutex::new(()),
        })
    }
    /// Connects to the first accessible reader containing a card.
    /// Non-presence failures are preserved if no connection succeeds.
    pub fn connect_first() -> Result<Self, Error> {
        Ok(Self {
            connection: Connection::connect_first()?,
            read_lock: Mutex::new(()),
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
    pub fn is_present(&self) -> Result<bool, Error> {
        self.connection.is_present()
    }
    /// Reads all supported public groups into an owned snapshot.
    pub fn read(&self) -> Result<EmiratesIdData, Error> {
        self.read_with_options(ReadOptions::all())
    }
    /// Reads identifiers and core identity plus the requested optional groups.
    /// Transport and malformed-data errors fail the read; inaccessible optional
    /// groups are represented by [`crate::ReadStatus`].
    pub fn read_with_options(&self, options: ReadOptions) -> Result<EmiratesIdData, Error> {
        let _lock = self.read_lock.lock().map_err(|_| {
            Error::new(
                ErrorKind::Protocol,
                "Session read was interrupted by a panic; reconnect",
            )
        })?;
        let _transaction = self.connection.begin_transaction()?;
        Reader {
            exchange: |command: &[u8]| self.connection.transmit(command),
        }
        .read(self.reader_name(), self.card_generation(), options)
    }
}
