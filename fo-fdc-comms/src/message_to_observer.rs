use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::Ammunition;

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageToObserver {
    src: String,
    receiver: String,
    target_number: TargetNumber,
    ammunition: Ammunition,
    rounds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TargetNumber {
    value: String,
}

impl TargetNumber {
    /// Constructs a new TargetNumber instance, returning an error if the input does not match target number standards:
    /// [A-Z]{2}\d{4}
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
