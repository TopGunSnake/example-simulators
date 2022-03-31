//! Container module for Battle Damage Assessment definitions
use serde::{Deserialize, Serialize};

/// A Battle Damage Assessment, providing feedback to an FDC of the effect of a fire mission
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub struct BattleDamageAssessment;
//TODO: Fill in the struct.
