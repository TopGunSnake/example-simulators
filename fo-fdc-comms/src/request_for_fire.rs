use std::net::SocketAddrV4;

use serde::{Deserialize, Serialize};

use crate::Ammunition;

/// A complete request for fire
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
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

/// Mission Types
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissionType {
    /// Adjust Fire mission variant. Represents a fire mission that needs to use a series of adjustments to dial in.
    AdjustFire,
    /// Fire For Effect mission variant. Once the FO confirms the FDC is accurate, and is certain that a full volley will have intended effect.
    FireForEffect,
}

/// Target Location Methods
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct TargetDescription {
    pub target_type: String,
    pub activity: String,
    pub numbers: String,
    pub protection: String,
}

/// Method of fire
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodOfFire {
    /// Indicates that the FO wants the FDC to wait for a commanded fire before beginning shots.
    AtMyCommand,
    /// Indicates a time-on-target request. The value will be the requested impact time in minutes past the hour.
    TimeOnTarget(u32),
}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_serde() {
        let call_for_fire = WarnOrder {
            src: "november".to_string(),
            receiver: "talon".to_string(),
            response_addr: SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080),
            mission_type: MissionType::AdjustFire,
            target_location: TargetLocation::Grid {
                lateral: 123,
                longitudinal: 456,
            },
            target_description: TargetDescription::default(),
            danger_close: false,
            ammunition: Some(Ammunition::HighExplosive),
            method_of_fire: Some(MethodOfFire::TimeOnTarget(13)),
        };

        let json = serde_json::to_string_pretty(&call_for_fire).unwrap();

        let verified: WarnOrder = serde_json::from_str(&json).unwrap();

        assert_eq!(call_for_fire, verified);
    }
}
