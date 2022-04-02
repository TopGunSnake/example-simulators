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
#[cfg_attr(test, derive(Clone))]
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
            0x03 => todo!(),
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
#[cfg_attr(test, derive(Clone))]
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

    use proptest::{collection::hash_map, prelude::*};

    fn fdc_gun_message_strategy() -> impl Strategy<Value = FdcGunMessage> {
        prop_oneof![
            Just(FdcGunMessage::CheckFire),
            Just(FdcGunMessage::StatusRequest),
            // Just(FdcGunMessage::FireReport),
            compliance_case(),
            // fire_command_case(),
            // status_reply_case(),
        ]
        .boxed()
    }

    fn compliance_strategy() -> impl Strategy<Value = Compliance> {
        prop_oneof![
            Just(Compliance::CANTCO),
            Just(Compliance::WILLCO),
            Just(Compliance::HAVECO),
        ]
    }

    prop_compose! {
        fn compliance_case()(
            compliance in compliance_strategy(),
        ) -> FdcGunMessage {
            FdcGunMessage::ComplianceResponse { compliance }
        }
    }

    fn ammunition_strategy() -> impl Strategy<Value = Ammunition> {
        prop_oneof![Just(Ammunition::HighExplosive),]
    }

    prop_compose! {
        fn fire_command_case()(
            rounds in any::<u32>(),
            ammunition in ammunition_strategy(),
            range in any::<u32>(),
            direction in any::<u32>(),
        ) -> FdcGunMessage {
            let target_location = TargetLocation { range, direction };
            FdcGunMessage::FireCommand {rounds, ammunition, target_location }
        }
    }

    fn status_strategy() -> impl Strategy<Value = Status> {
        prop_oneof![
            Just(Status::NonOperational),
            Just(Status::PartialOperational),
            Just(Status::Operational),
        ]
    }

    prop_compose! {
        fn status_reply_case()(
            status in status_strategy(),
            rounds in hash_map(ammunition_strategy(), any::<u32>(), 0..100)
        ) -> FdcGunMessage {
            FdcGunMessage::StatusReply {status, rounds}
        }
    }

    proptest! {
        #[test]
        fn test_serialize_deserialize(message in fdc_gun_message_strategy()) {
            let mut bytes = Vec::new();
            message.serialize(&mut bytes).unwrap();

            let output = FdcGunMessage::deserialize(bytes.as_slice()).unwrap();

            assert_eq!(message, output);
        }
    }
}
