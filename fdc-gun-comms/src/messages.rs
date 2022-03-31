//! Contains all possible strongly-typed messages for FDC-Gun Communication over TCP
//!
//! Each message definition can be converted to [`FdcGunMessage`] using [`Into`], and
//! from [`FdcGunMessage`] to a specific message via [`TryInto`].
//!
//! [`FdcGunMessage`]: super::FdcGunMessage

pub mod convert;
pub mod ops;

#[derive(Debug, Default)]
pub struct StatusRequest {}

#[derive(Debug, Default)]
pub struct StatusReply {}

#[derive(Debug, Default)]
pub struct FireReport {}

#[derive(Debug, Default)]
pub struct FireCommand {}

#[derive(Debug, Default)]
pub struct CheckFire {}

#[derive(Debug, Default)]
pub struct ComplianceResponse {}
