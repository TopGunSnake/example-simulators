//! Provides the message definitions and utilities for the FDC - Gun comm interface
//!
//! Since the interface is over TCP, the message model is as follows:
//! Raw Bytes (`Vec<u8>`) <-> [`FdcGunMessage`] with bytes and a message ID,
//! and finally specific message instances with respective strong types.

use std::{
    collections::HashMap,
    io::{self, Write},
};

use byteorder::{NetworkEndian, WriteBytesExt};
use num_enum::{IntoPrimitive, TryFromPrimitive};

/// High-level message definition.
///
/// Intended to be used as an intermediate between raw bytes and a specific strongly typed message
#[derive(Debug)]
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
            FdcGunMessage::StatusRequest => 0x02,
            FdcGunMessage::StatusReply { .. } => 0x03,
            FdcGunMessage::FireReport => 0x01,
            FdcGunMessage::FireCommand { .. } => 0x05,
            FdcGunMessage::CheckFire => 0x06,
            FdcGunMessage::ComplianceResponse { .. } => 0x00,
        }
    }
}

impl FdcGunMessage {
    pub fn serialize(&self, buf: &mut (impl Write)) -> io::Result<()> {
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
}
/// Ammunition types
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Ammunition {
    HighExplosive,
}

/// Gun status
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
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
#[derive(Debug)]
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
#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Compliance {
    CANTCO = 0x01,
    WILLCO = 0x02,
    HAVECO = 0x03,
}
