#![warn(
    missing_docs,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    // clippy::missing_panics_doc, // Turn this back on once we get the todo!() handled
    clippy::undocumented_unsafe_blocks
)]
//! Provides the message definitions and utilities for the FDC - Gun comm interface
//!
//! Since the interface is over TCP, the message model is as follows:
//! Raw Bytes (`Vec<u8>`) <-> [`FdcGunMessage`] with bytes and a message ID,
//! and finally specific message instances with respective strong types.

use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};
use itertools::Itertools;
use num_enum::{IntoPrimitive, TryFromPrimitive};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// High-level message definition.
///
/// Intended to be used as an intermediate between raw bytes and a specific strongly typed message
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
#[cfg_attr(test, derive(Arbitrary))]
pub enum FdcGunMessage {
    /// A request for status
    StatusRequest,

    /// A reply for a status request
    StatusReply {
        /// High-level status
        status: Status,
        /// Rounds available
        rounds: HashMap<Ammunition, u32>,
    },

    /// A Report of gun fires
    #[cfg_attr(test, proptest(skip))]
    FireReport,

    /// A Command from the FDC to fire
    #[cfg_attr(test, proptest(skip))]
    FireCommand {
        /// Number of rounds to fire
        rounds: u32,
        /// Type of ammunition for fires
        ammunition: Ammunition,
        /// Location of target
        target_location: TargetLocation,
    },

    /// A Check Fire command to a gun, to stop any active fires
    CheckFire,

    /// A Compliance response to a CheckFire, FireCommand
    ComplianceResponse {
        /// The specific compliance type
        compliance: Compliance,
    },
}

impl From<&FdcGunMessage> for u8 {
    /// Helper conversion to get the type ID for a [`FdcGunMessage`]
    fn from(msg: &FdcGunMessage) -> Self {
        match msg {
            FdcGunMessage::ComplianceResponse { .. } => 0x00,
            FdcGunMessage::FireReport => 0x01,
            FdcGunMessage::StatusRequest => 0x02,
            FdcGunMessage::StatusReply { .. } => 0x03,
            FdcGunMessage::FireCommand { .. } => 0x05,
            FdcGunMessage::CheckFire => 0x06,
        }
    }
}

impl FdcGunMessage {
    /// Serializes an [`FdcGunMessage`] to the supplied buffer.
    ///
    /// # Errors
    ///
    /// This method returns a [`std::io::Error`] if there is data missing,
    /// or if any data is otherwise invalid
    pub fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        let message_contents = {
            let mut message_contents = Vec::new();
            match self {
                FdcGunMessage::StatusRequest => (),
                FdcGunMessage::StatusReply { status, rounds } => {
                    message_contents.write_u8((*status).into())?;

                    for (ammo_type, ammo_count) in rounds {
                        message_contents.write_u8((*ammo_type).into())?;
                        message_contents.write_u32::<NetworkEndian>(*ammo_count)?;
                    }
                }
                FdcGunMessage::FireReport => todo!(),
                FdcGunMessage::FireCommand {
                    rounds,
                    ammunition,
                    target_location,
                } => {
                    message_contents.write_u32::<NetworkEndian>(*rounds)?;
                    message_contents.write_u8((*ammunition).into())?;
                    target_location.serialize(&mut message_contents)?;
                }
                FdcGunMessage::CheckFire => (),
                FdcGunMessage::ComplianceResponse { compliance } => {
                    message_contents.write_u8((*compliance).into())?;
                }
            }
            message_contents
        };

        // Write header
        let message_length = message_contents
            .len()
            .try_into()
            .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;

        buf.write_u32::<NetworkEndian>(message_length)?;
        buf.write_u8(self.into())?;

        // Write data
        buf.write_all(&message_contents)?;

        Ok(())
    }
    /// Deserializes an [`FdcGunMessage`] from the supplied buffer.
    ///
    /// # Errors
    ///
    /// This method returns a [`std::io::Error`] if there is data missing,
    /// or if any data is otherwise invalid
    pub fn deserialize(mut buf: impl Read) -> io::Result<Self> {
        let _message_len = buf.read_u32::<NetworkEndian>()?;

        match buf.read_u8()? {
            // ComplianceResponse
            0x00 => {
                let compliance = buf
                    .read_u8()?
                    .try_into()
                    .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;
                Ok(FdcGunMessage::ComplianceResponse { compliance })
            }
            // FireReport
            0x01 => todo!(),
            // StatusRequest
            0x02 => Ok(FdcGunMessage::StatusRequest),
            // StatusReply
            0x03 => {
                let status = buf
                    .read_u8()?
                    .try_into()
                    .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;
                let rounds_chunks = buf.bytes().chunks(5);
                let mut rounds: HashMap<Ammunition, u32> = HashMap::new();
                for mut chunk in &rounds_chunks {
                    let ammunition = chunk
                        .next()
                        .ok_or_else(|| io::Error::from(io::ErrorKind::InvalidData))??
                        .try_into()
                        .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;

                    let count = chunk.collect::<Result<Vec<u8>, _>>()?;
                    let count: [u8; 4] = count.try_into().map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidData, "Not enough data")
                    })?;
                    let count = u32::from_be_bytes(count);
                    rounds.insert(ammunition, count);
                }
                Ok(FdcGunMessage::StatusReply { status, rounds })
            }
            // FireCommand
            0x05 => todo!(),
            // CheckFire
            0x06 => Ok(FdcGunMessage::CheckFire),

            // Unsupported types
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Message Type",
            )),
        }
    }
}

/// Ammunition types
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, Hash)]
#[cfg_attr(test, derive(Arbitrary))]
#[repr(u8)]
pub enum Ammunition {
    /// HE rounds
    HighExplosive = 0x00,
}

/// Gun status
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
#[repr(u8)]
pub enum Status {
    /// Non-operational if no ammunition or other mission-critical fault
    NonOperational = 0x00,
    /// Partial operational capability, if less than minimum ammunition counts, or non-mission-critical fault
    PartialOperational = 0x01,
    /// Full operational status
    Operational = 0x02,
}

/// A gun's aim
#[derive(Debug, PartialEq)]
#[cfg_attr(test, derive(Clone))]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TargetLocation {
    /// Range in meters
    range: u32,
    /// Direction in mils
    direction: u32,
}

impl TargetLocation {
    /// Serializes a [`TargetLocation`] to the supplied buffer
    fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u32::<NetworkEndian>(self.range)?;
        buf.write_u32::<NetworkEndian>(self.direction)?;
        Ok(())
    }

    /// Deserializes a [`TargetLocation`] from the supplied buffer
    fn deserialize(mut buf: impl Read) -> io::Result<Self> {
        let range = buf.read_u32::<NetworkEndian>()?;
        let direction = buf.read_u32::<NetworkEndian>()?;

        Ok(Self { range, direction })
    }
}

/// Compliance types
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
#[repr(u8)]
pub enum Compliance {
    /// Indicates that the system cannot comply with the command
    CANTCO = 0x01,
    /// Indicates the system will comply with the command
    WILLCO = 0x02,
    /// Indicates the system has already complied with the command
    HAVECO = 0x03,
}

#[cfg(test)]
mod tests {
    use super::*;

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serialize_deserialize(message in any::<FdcGunMessage>()) {
            let mut bytes = Vec::new();
            message.serialize(&mut bytes).unwrap();

            let output = FdcGunMessage::deserialize(bytes.as_slice()).unwrap();

            assert_eq!(message, output);
        }
    }
}
