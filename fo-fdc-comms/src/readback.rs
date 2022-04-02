//! Container module for Solid Readback definitions
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// A readback confirmation message
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct SolidReadback;

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_serde(message in any::<SolidReadback>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: SolidReadback = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified);
        }
    }
}
