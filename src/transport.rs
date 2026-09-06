//! Native PC/SC transport: WinSCard, pcsc-lite, or Apple's PCSC framework.
use crate::apdu::exchange_apdu;
use crate::{CardGeneration, Error, ErrorKind};
use pcsc::{Context, Disposition, Protocols, Scope, ShareMode};
use std::ffi::{CStr, CString};
use std::sync::{Mutex, MutexGuard};

// Preserve LeaveCard semantics instead of pcsc::Card's default reset-on-drop.
struct CardHandle(Option<pcsc::Card>);
impl Drop for CardHandle {
    fn drop(&mut self) {
        if let Some(card) = self.0.take() {
            let _ = card.disconnect(Disposition::LeaveCard);
        }
    }
}

pub(crate) struct Connection {
    // pcsc::Card internally retains its parent context until it is dropped.
    card: Mutex<CardHandle>,
    atr: Vec<u8>,
    reader_name: String,
}

fn native_error(operation: &str, error: pcsc::Error) -> Error {
    Error::pcsc(operation, error)
}
fn context() -> Result<Context, Error> {
    Context::establish(Scope::User).map_err(|error| native_error("SCardEstablishContext", error))
}
fn readers(context: &Context) -> Result<Vec<CString>, Error> {
    match context.list_readers_owned() {
        Ok(readers) => Ok(readers),
        Err(pcsc::Error::NoReadersAvailable) => Ok(Vec::new()),
        Err(error) => Err(native_error("SCardListReaders", error)),
    }
}

impl Connection {
    pub(crate) fn reader_names() -> Result<Vec<String>, Error> {
        Ok(readers(&context()?)?
            .iter()
            .map(|reader| reader.to_string_lossy().into_owned())
            .collect())
    }
    pub(crate) fn connect(reader_name: &str) -> Result<Self, Error> {
        let name = CString::new(reader_name)
            .map_err(|_| Error::new(ErrorKind::InvalidArgument, "Reader name contains NUL"))?;
        Self::connect_reader(&context()?, &name)
    }
    fn connect_reader(context: &Context, name: &CStr) -> Result<Self, Error> {
        let card = context
            .connect(name, ShareMode::Shared, Protocols::T0 | Protocols::T1)
            .map_err(|error| native_error("SCardConnect", error))?;
        let handle = CardHandle(Some(card));
        let atr = handle
            .0
            .as_ref()
            .expect("new card handle")
            .status2_owned()
            .map_err(|error| native_error("SCardStatus", error))?
            .atr()
            .to_vec();
        Ok(Self {
            card: Mutex::new(handle),
            atr,
            reader_name: name.to_string_lossy().into_owned(),
        })
    }
    pub(crate) fn connect_first() -> Result<Self, Error> {
        let context = context()?;
        let readers = readers(&context)?;
        if readers.is_empty() {
            return Err(Error::new(
                ErrorKind::NoReader,
                "No PC/SC reader is available",
            ));
        }
        let mut connection_error = None;
        for name in readers {
            match Self::connect_reader(&context, &name) {
                Ok(connection) => return Ok(connection),
                Err(error) if error.kind == ErrorKind::NoCard => {}
                Err(error) => {
                    connection_error.get_or_insert(error);
                }
            }
        }
        Err(connection_error
            .unwrap_or_else(|| Error::new(ErrorKind::NoCard, "No inserted smart card was found")))
    }
    fn lock(&self) -> Result<MutexGuard<'_, CardHandle>, Error> {
        self.card.lock().map_err(|_| {
            Error::new(
                ErrorKind::Protocol,
                "Session operation was interrupted by a panic; reconnect",
            )
        })
    }
    pub(crate) fn with_transaction<T>(
        &self,
        operation: impl FnOnce(&pcsc::Transaction<'_>) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let mut guard = self.lock()?;
        let card = guard.0.as_mut().expect("live connection retains card");
        let transaction = card
            .transaction()
            .map_err(|error| native_error("SCardBeginTransaction", error))?;
        let result = operation(&transaction);
        // Preserve read errors; report an end failure after an otherwise successful read.
        let end = transaction
            .end(Disposition::LeaveCard)
            .map_err(|(_, error)| native_error("SCardEndTransaction", error));
        match result {
            Ok(value) => end.map(|()| value),
            Err(error) => Err(error),
        }
    }
    pub(crate) fn close(self) -> Result<(), Error> {
        // Recover a poisoned mutex to still release its native handle.
        let mut handle = self
            .card
            .into_inner()
            .unwrap_or_else(|error| error.into_inner());
        match handle.0.take() {
            Some(card) => card
                .disconnect(Disposition::LeaveCard)
                .map_err(|(_, error)| native_error("SCardDisconnect", error)),
            None => Ok(()),
        }
    }
    pub(crate) fn reader_name(&self) -> &str {
        &self.reader_name
    }
    pub(crate) fn atr(&self) -> &[u8] {
        &self.atr
    }
    pub(crate) fn atr_hex(&self) -> String {
        self.atr
            .iter()
            .map(|byte| format!("{byte:02X}"))
            .collect::<Vec<_>>()
            .join(" ")
    }
    pub(crate) fn card_generation(&self) -> CardGeneration {
        CardGeneration::from_atr(&self.atr)
    }
    pub(crate) fn is_present(&self) -> Result<bool, Error> {
        let guard = self.lock()?;
        match guard
            .0
            .as_ref()
            .expect("live connection retains card")
            .status2_owned()
        {
            Ok(status) => Ok(!status.status().contains(pcsc::Status::ABSENT)),
            Err(error) => {
                let error = native_error("SCardStatus", error);
                match error.kind {
                    ErrorKind::NoCard | ErrorKind::CardRemoved | ErrorKind::NoReader => Ok(false),
                    _ => Err(error),
                }
            }
        }
    }
}

pub(crate) fn transmit(card: &pcsc::Card, request: &[u8]) -> Result<Vec<u8>, Error> {
    let mut buffer = [0; pcsc::MAX_BUFFER_SIZE];
    exchange_apdu(request, |command| {
        let response = card
            .transmit(command, &mut buffer)
            .map_err(|error| native_error("SCardTransmit", error))?;
        if response.len() < 2 {
            return Err(Error::new(
                ErrorKind::Protocol,
                "Card response has no status word",
            ));
        }
        let end = response.len() - 2;
        Ok((
            response[..end].to_vec(),
            u16::from_be_bytes([response[end], response[end + 1]]),
        ))
    })
}
