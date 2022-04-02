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
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// High-level message definition.
///
/// Intended to be used as an intermediate between raw bytes and a specific strongly typed message
#[derive(Debug, PartialEq)]
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
    FireReport,

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
    pub fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u8(self.into())?;

        match self {
            FdcGunMessage::StatusRequest => (),
            FdcGunMessage::StatusReply { status, rounds } => {
                buf.write_u8((*status).into())?;

                for (ammo_type, ammo_count) in rounds {
                    buf.write_u8((*ammo_type).into())?;
                    buf.write_u32::<NetworkEndian>(*ammo_count)?;
                }
            }
            FdcGunMessage::FireReport => todo!(),
            FdcGunMessage::FireCommand {
                rounds,
                ammunition,
                target_location,
            } => {
                buf.write_u32::<NetworkEndian>(*rounds)?;
                buf.write_u8((*ammunition).into())?;
                target_location.serialize(buf)?;
            }
            FdcGunMessage::CheckFire => (),
            FdcGunMessage::ComplianceResponse { compliance } => {
                buf.write_u8((*compliance).into())?;
            }
        }
        Ok(())
    }

    pub fn deserialize(mut buf: impl Read) -> io::Result<Self> {
        match buf.read_u8()? {
            0x00 => todo!(),
            0x01 => todo!(),
            0x02 => todo!(),
            0x03 => Ok(FdcGunMessage::StatusRequest),
            0x05 => todo!(),
            0x06 => Ok(FdcGunMessage::CheckFire),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Invalid Message Type",
            )),
        }
    }
}
/// Ammunition types
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ammunition {
    HighExplosive,
}

/// Gun status
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum Status {
    /// Non-operational if no ammunition or other mission-critical fault
    NonOperational,
    /// Partial operational capability, if less than minimum ammunition counts, or non-mission-critical fault
    PartialOperational,
    /// Full operational status
    Operational,
}

/// A gun's aim
#[derive(Debug, PartialEq)]
pub struct TargetLocation {
    /// Range in meters
    range: u32,
    /// Direction in mils
    direction: u32,
}

impl TargetLocation {
    pub fn serialize(&self, buf: &mut impl Write) -> io::Result<()> {
        buf.write_u32::<NetworkEndian>(self.range)?;
        buf.write_u32::<NetworkEndian>(self.direction)?;
        Ok(())
    }
}

/// Compliance types
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum Compliance {
    CANTCO = 0x01,
    WILLCO = 0x02,
    HAVECO = 0x03,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let message = FdcGunMessage::CheckFire;
        let mut bytes = Vec::new();
        message.serialize(&mut bytes).unwrap();

        let output = FdcGunMessage::deserialize(bytes.as_slice()).unwrap();

        assert_eq!(message, output);
    }
}
