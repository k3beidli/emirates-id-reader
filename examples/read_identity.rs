//! Read identity without transferring images or extended data.
use emirates_id_reader::{CardSession, Language, ReadOptions};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session = CardSession::connect_first()?;
    let card = session.read_with_options(ReadOptions::identity_only())?;
    // Bind these borrowed values to your UI; avoid logging personal data.
    let _name = card.get_name();
    let _arabic_name = card.get_name_in(Language::Arabic);
    let _id = card.get_id_number();
    println!("Identity read successfully on {}", session.reader_name());
    Ok(())
}
