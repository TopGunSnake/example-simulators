//! Container module for Message to Observer definitions

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Ammunition;

/// The Message to Observer (MTO), sent by a FDC once the FDC has a response to an FO's RFF
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MessageToObserver {
    /// The sender's callsign
    pub src: String,
    /// The intended receiver's callsign
    pub receiver: String,
    /// The target number for this fire mission
    pub target_number: TargetNumber,
    /// The Ammunition in effect for this fire mission
    pub ammunition: Ammunition,
    /// The number of rounds (a volley) for the fire mission
    ///
    /// The total number of rounds sent down range depend on the gun systems executing the fire mission
    pub rounds: u32,
}

/// Wrapper type for Target Numbers
///
/// All Target Numbers, when viewed as a string, match the regex `r"[A-Z]{2}\d{4}$"`, for example: AN2001.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TargetNumber {
    /// The underlying string
    value: String,
}

impl TargetNumber {
    /// Constructs a new TargetNumber instance, returning an error if the input does not match target number standards:
    /// [A-Z]{2}\d{4}
    ///
    /// # Arguments
    /// * `input` - a target number in raw format
    pub fn new(input: &str) -> Result<Self, &str> {
        let regex_pattern: Regex = regex::Regex::new(r"^[A-Z]{2}\d{4}$").unwrap();
        if regex_pattern.is_match(input) {
            Ok(Self {
                value: input.to_string(),
            })
        } else {
            Err("Invalid target number")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_numbers() {
        let good_number = "AN2001";
        let bad_number = "A1N200";

        let good_target = TargetNumber::new(good_number);
        let bad_target = TargetNumber::new(bad_number);

        assert!(good_target.is_ok());
        assert!(bad_target.is_err());
    }
}
