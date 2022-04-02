//! Container module for Solid Readback definitions
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// A readback confirmation message
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "tag", rename = "solid_readback")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct SolidReadback {}

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    #[test]
    #[ignore = "Available for debugging only"]
    fn view_json() {
        let message = SolidReadback {};

        let json = serde_json::to_string_pretty(&message).unwrap();
        println!("{message:?} : {json}");
    }

    proptest! {
        #[test]
        fn test_serde(message in any::<SolidReadback>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: SolidReadback = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified, "{}", json);
        }
    }
}
