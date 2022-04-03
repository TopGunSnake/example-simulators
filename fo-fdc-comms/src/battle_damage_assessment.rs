//! Container module for Battle Damage Assessment definitions
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// A Battle Damage Assessment, providing feedback to an FDC of the effect of a fire mission
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct BattleDamageAssessment {}
//TODO: Fill in the struct.

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    #[test]
    #[ignore = "Available for debugging only"]
    fn view_json() {
        let message = BattleDamageAssessment {};

        let json = serde_json::to_string_pretty(&message).unwrap();
        println!("{message:?} : {json}");
    }

    proptest! {
        #[test]
        fn test_serde(message in any::<BattleDamageAssessment>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: BattleDamageAssessment = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified, "{}", json);
        }
    }
}
