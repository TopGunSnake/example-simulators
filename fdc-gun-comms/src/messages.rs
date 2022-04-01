//! Contains all possible strongly-typed messages for FDC-Gun Communication over TCP
//!
//! Each message definition can be converted to [`FdcGunMessage`] using [`Into`], and
//! from [`FdcGunMessage`] to a specific message via [`TryInto`].
//!
//! [`FdcGunMessage`]: super::FdcGunMessage

use std::collections::HashMap;

pub mod compliance;
pub mod convert;
pub mod ops;
pub mod status;

use compliance::Compliance;
use status::{Ammunition, Status};

/// A Request for status
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatusRequest {}

/// A Reply to a status request
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatusReply {
    /// High-level status
    status: Status,
    /// Rounds available
    rounds: HashMap<Ammunition, u32>,
}

#[derive(Debug, Default)]
pub struct FireReport {}

#[derive(Debug, Default)]
pub struct FireCommand {}

/// A Check Fire command
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CheckFire {}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ComplianceResponse {
    compliance: Compliance,
}
