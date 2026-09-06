use std::io::Cursor;
use tiff::decoder::{Decoder, DecodingResult};

/// Convert TIFF signatures to a browser-compatible image, entirely in memory.
/// The library payload stays untouched. Unsupported images keep the UI fallback.
pub fn preview(bytes: &[u8]) -> Option<Vec<u8>> {
    if !bytes.starts_with(b"MM\0*") && !bytes.starts_with(b"II*\0") {
        return None;
    }
    let mut decoder = Decoder::new(Cursor::new(bytes)).ok()?;
    let (width, height) = decoder.dimensions().ok()?;
    if width == 0 || height == 0 || width > 4096 || height > 4096 {
        return None;
    }
    let depth = match decoder.colortype().ok()? {
        tiff::ColorType::Gray(1) => png::BitDepth::One,
        tiff::ColorType::Gray(8) => png::BitDepth::Eight,
        _ => return None,
    };
    let DecodingResult::U8(pixels) = decoder.read_image().ok()? else {
        return None;
    };
    let mut output = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut output, width, height);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(depth);
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }
    Some(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn converts_synthetic_tiff_and_rejects_invalid_payloads() {
        let mut input = Cursor::new(Vec::new());
        tiff::encoder::TiffEncoder::new(&mut input)
            .unwrap()
            .write_image::<tiff::encoder::colortype::Gray8>(2, 1, &[0, 255])
            .unwrap();
        let result = preview(input.get_ref()).unwrap();
        let mut decoder = png::Decoder::new(Cursor::new(result)).read_info().unwrap();
        let mut pixels = vec![0; decoder.output_buffer_size().unwrap()];
        decoder.next_frame(&mut pixels).unwrap();
        assert_eq!(pixels, [0, 255]);
        assert!(preview(b"MM\0*bad").is_none());
        assert!(preview(b"unsupported").is_none());
    }
}
