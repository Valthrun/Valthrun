use std::{
    io::Read,
    string::FromUtf8Error,
};

use byteorder::ReadBytesExt;

pub fn read_cstring<R, E>(reader: &mut R) -> Result<String, E>
where
    R: Read,
    E: From<FromUtf8Error> + From<std::io::Error>,
{
    let mut buffer = Vec::with_capacity(128);
    loop {
        let byte = reader.read_u8()?;
        if byte == 0x00 {
            break;
        }

        buffer.push(byte);
    }

    Ok(String::from_utf8(buffer)?)
}
