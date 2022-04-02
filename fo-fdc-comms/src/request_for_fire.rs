//! Container module for Request for Fire (RFF)
use std::net::SocketAddrV4;

use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

use crate::Ammunition;

/// A complete Request for Fire, the first message sent by a FO
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct WarnOrder {
    /// Callsign for the sender of this warning order
    pub src: String,
    /// Callsign for the intended receiver of this warning order
    pub receiver: String,
    /// The address that the FDC will send all traffic to the FO via
    pub response_addr: SocketAddrV4,
    /// The type of mission for this warning order
    pub mission_type: MissionType,
    /// The target location and method of locating
    pub target_location: TargetLocation,
    /// The description of the target
    pub target_description: TargetDescription,
    /// Is the target danger close
    pub danger_close: bool,
    /// Ammunition Type
    pub ammunition: Option<Ammunition>,
    /// Method of fire
    pub method_of_fire: Option<MethodOfFire>,
}

/// Potential Mission Types for a Request for Fire
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub enum MissionType {
    /// Represents a fire mission that needs to use a series of adjustments to dial in.
    AdjustFire,
    /// Represents a fire mission that does not need any adjustment.
    /// Usually used once the FO confirms the FDC is accurate,
    /// and is certain that a full volley will have intended effect.
    FireForEffect,
}

/// Target Location Methods
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub enum TargetLocation {
    /// A grid target message
    Grid {
        /// 3 to 5 digits representing the lateral grid number
        lateral: u32,
        /// 3 to 5 digits (same as lateral) representing the longitudinal grid number
        longitudinal: u32,
    },
    /// A polar target message
    Polar {
        /// On Target direction in mils grid
        direction: u32,
        /// On Target distance in meters
        distance: u32,
    },
}

/// A description of the target, for human interpretation. Not all fields are provided, and may be empty.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct TargetDescription {
    /// Type of target, e.g. "tanks, infantry"
    pub target_type: String,
    /// Target activity, e.g. "staging area" or "moving down MSR"
    pub activity: String,
    /// Number of Targets, e.g. "10 tanks, several battalions"
    pub numbers: String,
    /// Protection of targets, e.g. "dug in" or "out in open"
    pub protection: String,
}

/// The Method of Fire requested by the FO
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub enum MethodOfFire {
    /// Indicates that the FO wants the FDC to wait for a commanded fire before beginning shots.
    AtMyCommand,
    /// Indicates a time-on-target request. The value will be the requested impact time in minutes past the hour.
    TimeOnTarget(u32),
}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serde(message in any::<WarnOrder>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: WarnOrder = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified, "{}", json);
        }
    }
}
