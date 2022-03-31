//! Container module for Solid Readback definitions
use serde::{Deserialize, Serialize};

/// A readback confirmation message
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SolidReadback;
