//! The message definitions for the interface between an FO and an FDC.
#![forbid(unused_imports)]

use battle_damage_assessment::BattleDamageAssessment;
use readback::SolidReadback;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest_derive::Arbitrary;

use message_to_observer::MessageToObserver;
use request_for_fire::WarnOrder;
use shot_fire::{RoundsComplete, Shot, Splash};

pub mod battle_damage_assessment;
pub mod message_to_observer;
pub mod readback;
pub mod request_for_fire;
pub mod shot_fire;

/// An enumeration over all possible message types.
///
/// This is intended to be used for serialization/deserialization where message context matters.
///
/// The underlying type in each enum can be used as well.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FoFdcMessage {
    /// A Request for Fire originating from a FO
    RequestForFire(WarnOrder),
    /// A readback originating from a FDC for a Request for Fire
    RequestForFireConfirm(WarnOrder),

    /// A Message to Observer originating from a FDC
    MessageToObserver(MessageToObserver),
    /// A readback originating from a FO for a Message to Observer
    MessageToObserverConfirm(MessageToObserver),

    /// A Shot originating from a FDC
    Shot(Shot),
    /// A readback originating from a FO for a Shot
    ShotConfirm(Shot),

    /// A Splash originating from a FDC
    Splash(Splash),
    /// A readback originating from a FO for a Splash
    SplashConfirm(Splash),

    /// A RoundsComplete originating from a FDC
    RoundsComplete(RoundsComplete),
    /// A readback originating from a FO for a RoundsComplete
    RoundsCompleteConfirm(RoundsComplete),

    /// A Battle Damage Assessment originating from a FO
    BattleDamageAssessment(BattleDamageAssessment),
    /// A readback originating from a FDC for a Battle Damage Assessment
    BattleDamageAssessmentConfirm(BattleDamageAssessment),

    /// Indicates a solid readback in response to any readback message.
    /// Can originate from a FDC or a FO.
    SolidReadback(SolidReadback),
}

/// Ammunition types
#[non_exhaustive]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(Arbitrary))]
pub enum Ammunition {
    HighExplosive,
}
