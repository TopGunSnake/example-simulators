//! Contains definitions of [`Compliance`]

use num_enum::{IntoPrimitive, TryFromPrimitive};

#[derive(Debug, Clone, Copy, PartialEq, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum Compliance {
    CANTCO = 0x01,
    WILLCO = 0x02,
    HAVECO = 0x03,
}

// Once #[derive(Default)] is stabilized, replace this:
impl Default for Compliance {
    fn default() -> Self {
        Self::CANTCO
    }
}
