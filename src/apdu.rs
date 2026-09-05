//! ISO 7816 response continuation and length correction.

use crate::{Error, ErrorKind};

pub(crate) fn exchange_apdu<F>(request: &[u8], mut exchange_once: F) -> Result<Vec<u8>, Error>
where
    F: FnMut(&[u8]) -> Result<(Vec<u8>, u16), Error>,
{
    let mut command = request.to_vec();
    let mut response_data = Vec::new();
    let mut corrected = false;

    for _ in 0..32 {
        let (data, status) = exchange_once(&command)?;
        match status {
            0x9000 | 0x6282 => {
                // 6282 is the ISO 7816 EOF warning: returned bytes are valid,
                // but the requested Le extended past the end of the file.
                response_data.extend_from_slice(&data);
                return Ok(response_data);
            }
            0x6100..=0x61FF => {
                response_data.extend_from_slice(&data);
                let available = status as u8; // 00 means 256 bytes in short APDUs.
                let cla = request.first().copied().unwrap_or(0x00) & 0xEF;
                command = vec![cla, 0xC0, 0x00, 0x00, available];
                corrected = false;
            }
            0x6C00..=0x6CFF if command.len() >= 5 && !corrected => {
                corrected = true;
                let le_index = if command.len() == 5 {
                    Some(4)
                } else {
                    let lc = command[4] as usize;
                    (command.len() == 6 + lc).then_some(command.len() - 1)
                };
                let Some(le_index) = le_index else {
                    return Err(Error::apdu(status));
                };
                command[le_index] = status as u8; // 00 means 256 bytes.
            }
            _ => return Err(Error::apdu(status)),
        }
    }

    Err(Error::new(
        ErrorKind::Protocol,
        "Card APDU exceeded the response-chaining limit",
    ))
}
