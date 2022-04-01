//! Contains all possible strongly-typed messages for FDC-Gun Communication over TCP
//!
//! Each message definition can be converted to [`FdcGunMessage`] using [`Into`], and
//! from [`FdcGunMessage`] to a specific message via [`TryInto`].
//!
//! [`FdcGunMessage`]: super::FdcGunMessage

use status::{Ammunition, Status};
use std::collections::HashMap;

pub mod convert;
pub mod ops;
pub mod status;

#[derive(Debug, Default)]
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

#[derive(Debug, Default)]
pub struct CheckFire {}

#[derive(Debug, Default)]
pub struct ComplianceResponse {}
