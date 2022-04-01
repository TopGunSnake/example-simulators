//! Contains status enumerations, such as Ammunition types, Gun status,

use num_enum::{IntoPrimitive, TryFromPrimitive};

/// The Ammunition types a Gun can fire
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Ammunition {
    HighExplosive,
}

/// The status of the gun
#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Status {
    /// Non-operational if no ammunition or other mission-critical fault
    NonOperational,
    /// Partial operational capability, if less than minimum ammunition counts, or non-mission-critical fault
    PartialOperational,
    /// Full operational status
    Operational,
}

//TODO: Once https://github.com/rust-lang/rust/issues/86985 is stabilized, switch to derive.
impl Default for Status {
    fn default() -> Self {
        Self::NonOperational
    }
}
