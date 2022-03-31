//! Container module for Shot and Splash messages
use serde::{Deserialize, Serialize};

/// A Shot message, used by an FDC to indicate that rounds have started going down range.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Shot;

/// A Splash message, used by an FDC about 7 seconds before expected impact to indicate that the rounds should start hitting.
///
/// Used by the FO to ensure attentions for observing, especially if grossly wrong.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct Splash;

/// A Rounds Complete message, used by an FDC about 7 seconds after the last rounds are expected to impact.
///
/// Used by the FO to know when it is safe to enter the target area, as well as start BDA.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct RoundsComplete;
