#![forbid(unused_imports)]

use serde::{Deserialize, Serialize};

pub mod message_to_observer;
pub mod request_for_fire;

/// Ammunition types
#[non_exhaustive]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ammunition {
    HighExplosive,
}
