//! Keep the session open to observe removal without rereading identity.
use emirates_id_reader::CardSession;
use std::{thread, time::Duration};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read()?;
    while session.is_present()? {
        thread::sleep(Duration::from_millis(200));
    }
    drop(card); // Also clear any copies held by your UI or application state.
    println!("Card removed. Reconnect with a new session after reinsertion.");
    Ok(())
}
