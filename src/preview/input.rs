use std::io::{self, Read};

use crate::preview::custom::CustomPreview;

const MAX_INPUT_BYTES: u64 = 1024 * 1024;

pub fn read(reader: impl Read) -> io::Result<CustomPreview> {
    let mut bytes = Vec::new();
    reader.take(MAX_INPUT_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("preview input exceeds {MAX_INPUT_BYTES} bytes"),
        ));
    }
    if bytes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "preview input is empty",
        ));
    }
    let input = str::from_utf8(&bytes)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(CustomPreview::parse(input))
}
