//! Provides the message definitions and utilities for the FDC - Gun comm interface
//!
//! Since the interface is over TCP, the message model is as follows:
//! Raw Bytes (`Vec<u8>`) <-> [`FdcGunMessage`] with bytes and a message ID,
//! and finally specific message instances with respective strong types.

/// High-level message definition.
///
/// Intended to be used as an intermediate between raw bytes and a specific strongly typed message
#[derive(Debug)]
pub struct FdcGunMessage {
    /// The message ID for this message
    message_id: FdcGunMessageId,
    /// The bytes that are carried with this message.
    /// NOTE: This field is limited to a length that will fit into a [`u32`]
    message_contents: Vec<u8>,
}

impl FdcGunMessage {
    /// Get the [`FdcGunMessageId`] for this message
    pub fn get_message_id(&self) -> FdcGunMessageId {
        self.message_id
    }
}

impl From<FdcGunMessage> for Vec<u8> {
    /// Converts a message into raw bytes
    ///
    /// # Panic
    /// Panics if the length of the message contents is larger than [`u32::MAX`]
    fn from(message: FdcGunMessage) -> Self {
        let message_id_byte = message.message_id as u8;
        let message_len = message.message_contents.len();

        let mut bytes = Vec::<u8>::with_capacity(4 + 1 + message_len);
        bytes.extend((u32::try_from(message_len).unwrap()).to_be_bytes());
        bytes.push(message_id_byte);
        bytes.extend(message.message_contents);

        bytes
    }
}

/// Message ID Values for FDC-Gun messages
///
/// Limited to one byte
#[derive(Debug, Clone, Copy)]
pub enum FdcGunMessageId {
    /// A request for a gun to report its status
    StatusRequest = 0x02,
    /// A reply to a status request
    StatusReply = 0x03,

    /// A report indicating a gun has started fires
    FireReport = 0x01,

    /// A command from an FDC to fire
    FireCommand = 0x05,
    /// A Check Fire command from an FDC
    CheckFire = 0x06,

    /// A general compliance response from a gun
    ComplianceResponse = 0x00,
}

pub mod messages;

#[cfg(test)]
mod tests {
    use crate::{FdcGunMessage, FdcGunMessageId};

    #[test]
    fn test_fdc_message_into_bytes() {
        let message = FdcGunMessage {
            message_id: FdcGunMessageId::FireReport,
            message_contents: vec![10u8; 100],
        };

        let bytes: Vec<u8> = message.into();

        assert_eq!(
            bytes[0..4],
            100_u32.to_be_bytes(),
            "Message length are first four bytes"
        );
        assert_eq!(
            bytes[4],
            FdcGunMessageId::FireReport as u8,
            "Message ID is fifth byte"
        );
        assert_eq!(bytes[5..], vec![10u8; 100], "Remaining bytes are message");
    }
}
