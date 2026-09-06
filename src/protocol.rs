//! Platform-independent public-file reading and application selection.
use crate::decode::{decode_modifiable, decode_non_modifiable, field, required_ascii_digits};
use crate::{
    CardGeneration, DataGroupStatus, EmiratesIdData, Error, ErrorKind, ModifiableData, ReadOptions,
    ReadStatus,
};
// Fixed chip file identifiers, not cardholder values. Personal data is decoded
// from the bytes returned by the connected card on each read.
const PUBLIC_DATA_DIRECTORY: u16 = 0x0200;
const IDENTITY_FILE: u16 = 0x0201;
const PHOTO_FILE: u16 = 0x0202;
const NON_MODIFIABLE_FILE: u16 = 0x0203;
const MODIFIABLE_FILE: u16 = 0x0205;
const HOLDER_SIGNATURE_FILE: u16 = 0x0207;

pub(crate) struct Reader<F> {
    pub(crate) exchange: F,
}
impl<F> Reader<F>
where
    F: FnMut(&[u8]) -> Result<Vec<u8>, Error>,
{
    fn transmit(&mut self, command: &[u8]) -> Result<Vec<u8>, Error> {
        (self.exchange)(command)
    }
    /// Reads public data while allowing callers to skip expensive optional groups.
    pub(crate) fn read(
        &mut self,
        reader_name: &str,
        generation: CardGeneration,
        options: ReadOptions,
    ) -> Result<EmiratesIdData, Error> {
        self.select_public_base()?;
        let identity = self.read_file(IDENTITY_FILE)?;
        let id_number = required_ascii_digits(&identity, 0xE101, "ID number", 15)?;
        let card_number = required_ascii_digits(&identity, 0xE102, "card number", 9)?;
        let non_modifiable = decode_non_modifiable(&self.read_file(NON_MODIFIABLE_FILE)?)?;
        let (photo, photo_status) = self.read_optional_file(PHOTO_FILE, options.photo)?;
        let (modifiable, modifiable_status) =
            self.read_optional_file(MODIFIABLE_FILE, options.modifiable_data)?;
        let (signature, signature_status) =
            self.read_optional_file(HOLDER_SIGNATURE_FILE, options.holder_signature_image)?;
        Ok(EmiratesIdData {
            reader_name: reader_name.to_owned(),
            card_generation: generation,
            id_number,
            card_number,
            photo_jpeg: image_field(photo.as_deref(), 0x6203, true)?,
            holder_signature_image: image_field(signature.as_deref(), 0x6732, false)?,
            non_modifiable,
            modifiable: match modifiable {
                Some(data) => decode_modifiable(&data)?,
                None => ModifiableData::default(),
            },
            read_status: ReadStatus {
                identity: DataGroupStatus::Read,
                photo: photo_status,
                non_modifiable: DataGroupStatus::Read,
                modifiable: modifiable_status,
                holder_signature_image: signature_status,
            },
        })
    }

    fn read_optional_file(
        &mut self,
        file_id: u16,
        requested: bool,
    ) -> Result<(Option<Vec<u8>>, DataGroupStatus), Error> {
        if !requested {
            return Ok((None, DataGroupStatus::NotRequested));
        }
        match self.read_file(file_id) {
            Ok(data) => Ok((Some(data), DataGroupStatus::Read)),
            Err(error) if matches!(error.status_word, Some(0x6982) | Some(0x6985)) => {
                Ok((None, DataGroupStatus::Protected))
            }
            Err(error) if matches!(error.status_word, Some(0x6A82) | Some(0x6A83)) => {
                Ok((None, DataGroupStatus::NotAvailable))
            }
            Err(error) => Err(error),
        }
    }

    fn select_public_base(&mut self) -> Result<(), Error> {
        self.select_application()?;
        match self.select_file(PUBLIC_DATA_DIRECTORY) {
            Ok(()) => Ok(()),
            Err(error) if matches!(error.status_word, Some(0x6A82) | Some(0x6A83)) => {
                self.select_application()?;
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    fn read_file(&mut self, file_id: u16) -> Result<Vec<u8>, Error> {
        // The selected DF remains current while sibling EFs are selected.
        self.select_file(file_id)?;
        self.read_selected_file()
    }

    fn read_selected_file(&mut self) -> Result<Vec<u8>, Error> {
        let mut data = self.read_binary(0, 0xFD)?;
        if data.len() < 4 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "Emirates ID file header is truncated",
            ));
        }
        let total = 4 + u16::from_be_bytes([data[2], data[3]]) as usize;
        if total > 16 * 1024 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unreasonable card file length: {total}"),
            ));
        }
        while data.len() < total {
            let chunk = self.read_binary(data.len(), (total - data.len()).min(0xFD))?;
            if chunk.is_empty() {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Empty READ BINARY chunk",
                ));
            }
            data.extend_from_slice(&chunk);
        }
        data.truncate(total);
        Ok(data)
    }
    fn select_application(&mut self) -> Result<(), Error> {
        const AID: [u8; 12] = [
            0xA0, 0x00, 0x00, 0x02, 0x43, 0x00, 0x13, 0x00, 0x00, 0x00, 0x01, 0x01,
        ];
        let mut command = vec![0x00, 0xA4, 0x04, 0x00, AID.len() as u8];
        command.extend_from_slice(&AID);
        command.push(0x00);
        self.transmit(&command)?;
        Ok(())
    }

    fn select_file(&mut self, file_id: u16) -> Result<(), Error> {
        let [high, low] = file_id.to_be_bytes();
        self.transmit(&[0x00, 0xA4, 0x00, 0x00, 0x02, high, low, 0x00])?;
        Ok(())
    }

    fn read_binary(&mut self, offset: usize, length: usize) -> Result<Vec<u8>, Error> {
        if offset > 0x7FFF || length == 0 || length > 0xFD {
            return Err(Error::new(
                ErrorKind::Protocol,
                "Invalid READ BINARY offset or length",
            ));
        }
        self.transmit(&[
            0x00,
            0xB0,
            ((offset >> 8) & 0x7F) as u8,
            (offset & 0xFF) as u8,
            length as u8,
        ])
    }
}

fn image_field(data: Option<&[u8]>, tag: u16, jpeg: bool) -> Result<Option<Vec<u8>>, Error> {
    let Some(data) = data else {
        return Ok(None);
    };
    let Some(bytes) = field(data, tag)?.filter(|bytes| !bytes.is_empty()) else {
        return Ok(None);
    };
    if jpeg && !bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Photograph payload is not JPEG",
        ));
    }
    Ok(Some(bytes.to_vec()))
}
