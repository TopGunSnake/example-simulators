//! Container module for Shot and Splash messages
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// A Shot message, used by an FDC to indicate that rounds have started going down range.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Shot;

/// A Splash message, used by an FDC about 7 seconds before expected impact to indicate that the rounds should start hitting.
///
/// Used by the FO to ensure attentions for observing, especially if grossly wrong.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct Splash;

/// A Rounds Complete message, used by an FDC about 7 seconds after the last rounds are expected to impact.
///
/// Used by the FO to know when it is safe to enter the target area, as well as start BDA.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub struct RoundsComplete;

#[cfg(test)]
mod tests {

    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_shot_serde(message in any::<Shot>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: Shot = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified);
        }

        #[test]
        fn test_splash_serde(message in any::<Splash>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: Splash = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified);
        }

        #[test]
        fn test_rounds_complete_serde(message in any::<RoundsComplete>()) {
            let json = serde_json::to_string_pretty(&message).unwrap();

            let verified: RoundsComplete = serde_json::from_str(&json).unwrap();

            assert_eq!(message, verified);
        }
    }
}
