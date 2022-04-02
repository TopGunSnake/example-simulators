#![warn(
    missing_docs,
    clippy::doc_markdown,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
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
    FireReport {
        /// The shot number for this report
        shot: u8,
        /// The total number of shots that the gun issuing the report
        /// will fire
        total_shots: u8,
        /// The Ammunition in use
        ammunition: Ammunition,
        /// The location of this shot
        target_location: TargetLocation,
        /// The time in seconds from the issue of this
        /// [`FdcGunMessage::FireReport`] until the round is expected
        /// to land
        time_to_target: u32,
    },

    /// A Command from the FDC to fire
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
            FdcGunMessage::FireReport { .. } => 0x01,
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
                    serialize_status_reply(&mut message_contents, status, rounds)?;
                }
                FdcGunMessage::FireReport {
                    shot,
                    total_shots,
                    ammunition,
                    target_location,
                    time_to_target,
                } => serialize_fire_report(
                    &mut message_contents,
                    shot,
                    total_shots,
                    ammunition,
                    target_location,
                    time_to_target,
                )?,
                FdcGunMessage::FireCommand {
                    rounds,
                    ammunition,
                    target_location,
                } => {
                    serialize_fire_command(
                        &mut message_contents,
                        rounds,
                        ammunition,
                        target_location,
                    )?;
                }
                FdcGunMessage::CheckFire => (),
                FdcGunMessage::ComplianceResponse { compliance } => {
                    serialize_compliance_response(&mut message_contents, compliance)?;
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
            0x00 => deserialize_compliance_response(buf),
            // FireReport
            0x01 => deserialize_fire_report(buf),
            // StatusRequest
            0x02 => Ok(FdcGunMessage::StatusRequest),
            // StatusReply
            0x03 => deserialize_status_reply(buf),
            // FireCommand
            0x05 => deserialize_fire_command(buf),
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

/// Serializes the fields of a [`FdcGunMessage::ComplianceResponse`]
fn serialize_compliance_response(
    message_contents: &mut Vec<u8>,
    compliance: &Compliance,
) -> Result<(), io::Error> {
    message_contents.write_u8((*compliance).into())?;
    Ok(())
}

/// Serializes the fields of a [`FdcGunMessage::FireCommand`]
fn serialize_fire_command(
    message_contents: &mut Vec<u8>,
    rounds: &u32,
    ammunition: &Ammunition,
    target_location: &TargetLocation,
) -> Result<(), io::Error> {
    message_contents.write_u32::<NetworkEndian>(*rounds)?;
    message_contents.write_u8((*ammunition).into())?;
    target_location.serialize(message_contents)?;
    Ok(())
}

/// Serializes the fields of a [`FdcGunMessage::StatusReply`]
fn serialize_status_reply(
    message_contents: &mut Vec<u8>,
    status: &Status,
    rounds: &HashMap<Ammunition, u32>,
) -> Result<(), io::Error> {
    message_contents.write_u8((*status).into())?;
    for (ammo_type, ammo_count) in rounds {
        message_contents.write_u8((*ammo_type).into())?;
        message_contents.write_u32::<NetworkEndian>(*ammo_count)?;
    }
    Ok(())
}

/// Serializes the fields of a [`FdcGunMessage::FireReport`]
fn serialize_fire_report(
    message_contents: &mut Vec<u8>,
    shot: &u8,
    total_shots: &u8,
    ammunition: &Ammunition,
    target_location: &TargetLocation,
    time_to_target: &u32,
) -> Result<(), io::Error> {
    message_contents.write_u8(*shot)?;
    message_contents.write_u8(*total_shots)?;
    message_contents.write_u8((*ammunition).into())?;
    target_location.serialize(message_contents)?;
    message_contents.write_u32::<NetworkEndian>(*time_to_target)?;
    Ok(())
}

/// Deserializes to a [`FdcGunMessage::FireCommand`]
fn deserialize_fire_command(mut buf: impl Read) -> Result<FdcGunMessage, io::Error> {
    let rounds = buf.read_u32::<NetworkEndian>()?;
    let ammunition: Ammunition = buf
        .read_u8()?
        .try_into()
        .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;
    let target_location = TargetLocation::deserialize(&mut buf)?;
    Ok(FdcGunMessage::FireCommand {
        rounds,
        ammunition,
        target_location,
    })
}

/// Deserializes to a [`FdcGunMessage::StatusReply`]
fn deserialize_status_reply(mut buf: impl Read) -> Result<FdcGunMessage, io::Error> {
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
        let count: [u8; 4] = count
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Not enough data"))?;
        let count = u32::from_be_bytes(count);
        rounds.insert(ammunition, count);
    }
    Ok(FdcGunMessage::StatusReply { status, rounds })
}

/// Deserializes to a [`FdcGunMessage::ComplianceResponse`]
fn deserialize_compliance_response(mut buf: impl Read) -> Result<FdcGunMessage, io::Error> {
    let compliance = buf
        .read_u8()?
        .try_into()
        .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;
    Ok(FdcGunMessage::ComplianceResponse { compliance })
}

/// Deserializes to a [`FdcGunMessage::FireReport`]
fn deserialize_fire_report(mut buf: impl Read) -> Result<FdcGunMessage, io::Error> {
    let shot = buf.read_u8()?;
    let total_shots = buf.read_u8()?;
    let ammunition = buf
        .read_u8()?
        .try_into()
        .map_err(|conv_err| io::Error::new(io::ErrorKind::InvalidData, conv_err))?;
    let target_location = TargetLocation::deserialize(&mut buf)?;
    let time_to_target = buf.read_u32::<NetworkEndian>()?;

    Ok(FdcGunMessage::FireReport {
        shot,
        total_shots,
        ammunition,
        target_location,
        time_to_target,
    })
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
    fn deserialize(buf: &mut impl Read) -> io::Result<Self> {
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
