//! Contains definitions of [`Compliance`]

#[derive(Debug, Clone, Copy, PartialEq)]
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

impl TryFrom<u8> for Compliance {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::CANTCO as u8 => Ok(Self::CANTCO),
            value if value == Self::WILLCO as u8 => Ok(Self::WILLCO),
            value if value == Self::HAVECO as u8 => Ok(Self::HAVECO),
            _ => Err(()),
        }
    }
}

impl From<Compliance> for u8 {
    fn from(status: Compliance) -> Self {
        status as u8
    }
}
