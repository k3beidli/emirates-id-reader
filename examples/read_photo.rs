//! Transfer identity and a photograph, without extended data or a signature.
use emirates_id_reader::{CardSession, ReadOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only().with_photo(true))?;
    match card.get_photo() {
        Some(jpeg) => println!("Photo available: {} bytes", jpeg.len()),
        None => println!("Photo absent; group outcome: {:?}", card.read_status.photo),
    }
    // Pass jpeg bytes to your UI's image decoder. The SDK does not write a file.
    Ok(())
}
