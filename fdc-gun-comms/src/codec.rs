//! Provides tokio-util definitions for a codec.

use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::FdcGunMessage;

const MAX_BYTES: usize = 8 * 1024; // 8 KB
const MESSAGE_LEN_SIZE: usize = 4; // 4 Bytes for the message length marker.

/// [`tokio_util::codec::Decoder`] for [`FdcGunMessage`]
#[derive(Default)]
pub struct FdcGunMessageDecoder {}

impl Decoder for FdcGunMessageDecoder {
    type Item = FdcGunMessage;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < MESSAGE_LEN_SIZE {
            // Not enough data to read the message length marker
            return Ok(None);
        }

        // Read the length marker
        let mut length_bytes = [0u8; MESSAGE_LEN_SIZE];
        length_bytes.copy_from_slice(&src[..MESSAGE_LEN_SIZE]);
        let length = u32::from_be_bytes(length_bytes) as usize;

        // Check that the length is not too large to avoid a DOS attack
        if length > MAX_BYTES {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Frame of length {} is too large.", length),
            ));
        }

        if src.len() < MESSAGE_LEN_SIZE + length {
            // A full message has not yet arrived.
            //
            // We reserve more space in th ebuffer. This is not strictly
            // necessary, but helps with performance.
            src.reserve(MESSAGE_LEN_SIZE + length - src.len());

            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        // Use advance to modify src such that it no longer contains this frame.
        // Get the data, from the last byte of the header (inclusive) to the end of the expected message
        let data = src[..MESSAGE_LEN_SIZE + 1 + length].to_vec();
        src.advance(MESSAGE_LEN_SIZE + 1 + length);

        // Convert the data into an FdcGunMessage, or fail if invalid.
        match FdcGunMessage::deserialize(data.as_slice()) {
            Ok(message) => Ok(Some(message)),
            Err(io_error) => Err(io_error),
        }
    }
}

/// [`tokio_util::codec::Encoder`] for [`FdcGunMessage`]
#[derive(Default)]
pub struct FdcGunMessageEncoder {}

impl Encoder<FdcGunMessage> for FdcGunMessageEncoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: FdcGunMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.reserve(MESSAGE_LEN_SIZE + 30);
        item.serialize(dst)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {

        #[test]
        fn test_codec(message in any::<FdcGunMessage>()) {
            let mut buffer = BytesMut::new();

            let mut encoder = FdcGunMessageEncoder::default();

            let mut decoder = FdcGunMessageDecoder::default();

            encoder.encode(message.clone(), &mut buffer).unwrap();

            let decoded = decoder.decode(&mut buffer).unwrap().unwrap();

            assert_eq!(message, decoded);
        }
    }
}
