use std::io::{Read, Write};

use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

pub const MAX_FRAME_BYTES: usize = 1024 * 1024;

#[derive(Debug, Error)]
pub enum FrameError {
    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("message encoding failed: {0}")]
    Encode(#[from] rmp_serde::encode::Error),
    #[error("message decoding failed: {0}")]
    Decode(#[from] rmp_serde::decode::Error),
    #[error("frame exceeds the {MAX_FRAME_BYTES} byte limit")]
    TooLarge,
}

pub fn write_frame<T: Serialize>(writer: &mut impl Write, value: &T) -> Result<(), FrameError> {
    let payload = rmp_serde::to_vec_named(value)?;

    if payload.len() > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }

    writer.write_all(&(payload.len() as u32).to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

pub fn read_frame<T: DeserializeOwned>(reader: &mut impl Read) -> Result<T, FrameError> {
    let mut length = [0_u8; 4];
    reader.read_exact(&mut length)?;
    let length = u32::from_le_bytes(length) as usize;

    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }

    let mut payload = vec![0_u8; length];
    reader.read_exact(&mut payload)?;
    Ok(rmp_serde::from_slice(&payload)?)
}

pub fn decode_frame<T: DeserializeOwned>(buffer: &mut Vec<u8>) -> Result<Option<T>, FrameError> {
    if buffer.len() < 4 {
        return Ok(None);
    }

    let length = u32::from_le_bytes(
        buffer[..4]
            .try_into()
            .expect("frame prefix always contains four bytes"),
    ) as usize;

    if length > MAX_FRAME_BYTES {
        return Err(FrameError::TooLarge);
    }

    if buffer.len() < length + 4 {
        return Ok(None);
    }

    let value = rmp_serde::from_slice(&buffer[4..length + 4])?;
    buffer.drain(..length + 4);
    Ok(Some(value))
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::{Command, Envelope, PROTOCOL_VERSION};

    use super::{decode_frame, read_frame, write_frame};

    #[test]
    fn frame_round_trips() {
        let expected = Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "request".to_owned(),
            body: Command::Hello,
        };
        let mut bytes = Vec::new();

        write_frame(&mut bytes, &expected).expect("frame should encode");
        let actual = read_frame(&mut Cursor::new(bytes)).expect("frame should decode");

        assert_eq!(expected, actual);
    }

    #[test]
    fn waits_for_a_complete_streamed_frame() {
        let expected = Envelope {
            protocol_version: PROTOCOL_VERSION,
            request_id: "request".to_owned(),
            body: Command::Hello,
        };
        let mut bytes = Vec::new();
        write_frame(&mut bytes, &expected).expect("frame should encode");
        let remaining = bytes.split_off(3);

        assert!(
            decode_frame::<Envelope<Command>>(&mut bytes)
                .unwrap()
                .is_none()
        );
        bytes.extend(remaining);

        assert_eq!(decode_frame(&mut bytes).unwrap(), Some(expected));
        assert!(bytes.is_empty());
    }
}
