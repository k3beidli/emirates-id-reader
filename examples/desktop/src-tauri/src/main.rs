#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod signature;

use emirates_id_reader::{CardSession, EmiratesIdData, Language, ReadOptions};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{Emitter, Manager, State};

const SUPERSEDED: &str = "This request was cleared or replaced by a newer request.";

// Inspect insertion without connecting, resetting the card, or reading identity.
#[tauri::command]
async fn card_present(reader: String) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let name = std::ffi::CString::new(reader).map_err(|_| "Invalid reader name.")?;
        let context = pcsc::Context::establish(pcsc::Scope::User).map_err(|e| e.to_string())?;
        let mut states = [pcsc::ReaderState::new(name, pcsc::State::UNAWARE)];
        context
            .get_status_change(Duration::ZERO, &mut states)
            .map_err(|e| e.to_string())?;
        let state = states[0].event_state();
        if state.intersects(pcsc::State::UNKNOWN | pcsc::State::UNAVAILABLE) {
            return Err("Reader unavailable. Reconnect it and refresh readers.".into());
        }
        Ok(state.contains(pcsc::State::PRESENT))
    })
    .await
    .map_err(|_| "Card detection task failed.".to_string())?
}

// Ordering is enforced on both sides of IPC. An old read/presence check must
// never restore data or clear a newer session after a user clears/reconnects.
struct SessionSlot<T> {
    request_id: u64,
    active: Option<T>,
}

impl<T> Default for SessionSlot<T> {
    fn default() -> Self {
        Self {
            request_id: 0,
            active: None,
        }
    }
}

impl<T> SessionSlot<T> {
    fn begin(&mut self, request_id: u64) -> Result<Option<T>, String> {
        if request_id <= self.request_id {
            return Err(SUPERSEDED.into());
        }
        self.request_id = request_id;
        Ok(self.active.take())
    }

    fn finish(&mut self, request_id: u64, value: T) -> Result<(), String> {
        if request_id != self.request_id {
            return Err(SUPERSEDED.into());
        }
        self.active = Some(value);
        Ok(())
    }

    fn remove(&mut self, request_id: u64) -> Option<T> {
        if request_id == self.request_id {
            self.active.take()
        } else {
            None
        }
    }
}

#[derive(Default)]
struct Service {
    slot: Mutex<SessionSlot<Arc<CardSession>>>,
    read_gate: Mutex<()>,
    stopped: AtomicBool,
}

impl Service {
    fn begin(&self, request_id: u64) -> Result<(), String> {
        let old = self
            .slot
            .lock()
            .map_err(|_| "Session state unavailable; restart the app.")?
            .begin(request_id)?;
        drop(old); // Native disconnect happens without holding the state mutex.
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadRequest {
    photo: bool,
    modifiable_data: bool,
    holder_signature_image: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DisplayValues {
    full_name_english: Option<String>,
    full_name_arabic: Option<String>,
    id_number: String,
    gender_code: Option<String>,
}

impl From<&EmiratesIdData> for DisplayValues {
    fn from(data: &EmiratesIdData) -> Self {
        Self {
            full_name_english: data.formatted_name_in(Language::English),
            full_name_arabic: data.formatted_name_in(Language::Arabic),
            id_number: data.formatted_id_number(),
            gender_code: data.gender().map(|gender| gender.code().to_owned()),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Scan {
    display: DisplayValues,
    request_id: u64,
    data: EmiratesIdData,
    elapsed_ms: u128,
    atr: String,
    signature_preview_png: Option<Vec<u8>>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Removed {
    request_id: u64,
    reason: String,
}

#[tauri::command]
async fn refresh_readers(
    request_id: u64,
    service: State<'_, Arc<Service>>,
) -> Result<Vec<String>, String> {
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service.begin(request_id)?;
        CardSession::reader_names().map_err(|error| error.to_string())
    })
    .await
    .map_err(|_| "Reader task failed; restart the app.".to_string())?
}

#[tauri::command]
async fn clear_session(request_id: u64, service: State<'_, Arc<Service>>) -> Result<(), String> {
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || service.begin(request_id))
        .await
        .map_err(|_| "Session task failed; restart the app.".to_string())?
}

#[tauri::command]
async fn read_card(
    request_id: u64,
    reader: String,
    options: ReadRequest,
    service: State<'_, Arc<Service>>,
) -> Result<Scan, String> {
    let service = Arc::clone(service.inner());
    tauri::async_runtime::spawn_blocking(move || {
        service.begin(request_id)?;
        let _gate = service
            .read_gate
            .lock()
            .map_err(|_| "Reader task unavailable; restart the app.")?;
        if service
            .slot
            .lock()
            .map_err(|_| "Session state unavailable.")?
            .request_id
            != request_id
        {
            return Err(SUPERSEDED.into());
        }
        let start = Instant::now();
        let session = Arc::new(CardSession::connect(&reader).map_err(|error| error.to_string())?);
        let data = session
            .read_with_options(
                ReadOptions::all()
                    .with_photo(options.photo)
                    .with_modifiable_data(options.modifiable_data)
                    .with_holder_signature_image(options.holder_signature_image),
            )
            .map_err(|error| error.to_string())?;
        if !session.is_present().map_err(|error| error.to_string())? {
            return Err("Card removed before the read completed.".into());
        }
        let atr = session.atr_hex();
        service
            .slot
            .lock()
            .map_err(|_| "Session state unavailable.")?
            .finish(request_id, session)?;
        let signature_preview_png = data.signature().and_then(signature::preview);
        Ok(Scan {
            display: DisplayValues::from(&data),
            request_id,
            signature_preview_png,
            data,
            elapsed_ms: start.elapsed().as_millis(),
            atr,
        })
    })
    .await
    .map_err(|_| "Reader task failed; restart the app.".to_string())?
}

fn start_monitor(service: Arc<Service>, app: tauri::AppHandle) {
    std::thread::spawn(move || {
        while !service.stopped.load(Ordering::Relaxed) {
            std::thread::sleep(Duration::from_millis(300));
            let active = service.slot.lock().ok().and_then(|slot| {
                slot.active
                    .clone()
                    .map(|session| (slot.request_id, session))
            });
            let Some((request_id, session)) = active else {
                continue;
            };
            let reason = match session.is_present() {
                Ok(true) => continue,
                Ok(false) => "Card removed. Personal data has been cleared.".to_owned(),
                Err(error) => format!("Reader disconnected: {error}"),
            };
            let removed = service
                .slot
                .lock()
                .ok()
                .and_then(|mut slot| slot.remove(request_id));
            if removed.is_some() {
                let _ = app.emit("card-removed", Removed { request_id, reason });
            }
        }
    });
}

fn main() {
    let service = Arc::new(Service::default());
    tauri::Builder::default()
        .manage(service)
        .invoke_handler(tauri::generate_handler![
            refresh_readers,
            card_present,
            clear_session,
            read_card
        ])
        .setup(|app| {
            start_monitor(
                Arc::clone(app.state::<Arc<Service>>().inner()),
                app.handle().clone(),
            );
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Unable to initialize Emirates ID Reader")
        .run(|app, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                app.state::<Arc<Service>>()
                    .stopped
                    .store(true, Ordering::Relaxed);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn display_helpers_preserve_raw_card_values() {
        use emirates_id_reader::{CardGeneration, NonModifiableData};
        let mut identity = NonModifiableData::default();
        identity.full_name_english = Some("SYNTHETIC,, HOLDER".into());
        identity.full_name_arabic = Some(", ,".into());
        identity.gender = Some("m".into());
        let data = EmiratesIdData::builder("000000000000000", "000000000")
            .reader_name("Synthetic reader")
            .card_generation(CardGeneration::V2)
            .identity(identity)
            .build()
            .unwrap();
        let display = DisplayValues::from(&data);
        assert_eq!(
            display.full_name_english.as_deref(),
            Some("SYNTHETIC HOLDER")
        );
        assert_eq!(display.full_name_arabic, None);
        assert_eq!(display.id_number, "000-0000-0000000-0");
        assert_eq!(display.gender_code.as_deref(), Some("M"));
        assert_eq!(data.name(), Some("SYNTHETIC,, HOLDER"));
        assert_eq!(data.gender_code(), Some("m"));
    }

    #[test]
    fn clear_prevents_a_late_read_from_restoring_data() {
        let mut slot = SessionSlot::default();
        slot.begin(1).unwrap();
        slot.begin(2).unwrap();
        assert!(slot.finish(1, "old card").is_err());
        assert!(slot.active.is_none());
    }
    #[test]
    fn stale_clear_and_presence_check_do_not_destroy_a_new_session() {
        let mut slot = SessionSlot::default();
        slot.begin(5).unwrap();
        slot.finish(5, "current card").unwrap();
        assert!(slot.begin(4).is_err());
        assert!(slot.remove(4).is_none());
        assert_eq!(slot.active, Some("current card"));
        assert_eq!(slot.remove(5), Some("current card"));
    }
}
