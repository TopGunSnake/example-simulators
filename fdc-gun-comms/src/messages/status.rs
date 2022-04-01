#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Ammunition {
    HighExplosive,
}

impl TryFrom<u8> for Ammunition {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::HighExplosive as u8 => Ok(Self::HighExplosive),
            _ => Err(()),
        }
    }
}

impl From<Ammunition> for u8 {
    fn from(status: Ammunition) -> Self {
        status as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(u8)]
pub enum Status {
    NonOperational,
    PartialOperational,
    Operational,
}

impl TryFrom<u8> for Status {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            value if value == Self::NonOperational as u8 => Ok(Self::NonOperational),
            value if value == Self::PartialOperational as u8 => Ok(Self::PartialOperational),
            value if value == Self::Operational as u8 => Ok(Self::Operational),
            _ => Err(()),
        }
    }
}

impl From<Status> for u8 {
    fn from(status: Status) -> Self {
        status as u8
    }
}

//TODO: Once https://github.com/rust-lang/rust/issues/86985 is stabilized, switch to derive.
impl Default for Status {
    fn default() -> Self {
        Self::NonOperational
    }
}
