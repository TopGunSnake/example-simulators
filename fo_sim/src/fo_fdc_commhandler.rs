//! Contains the message types that the FO sim can send/receive, as well as the communication tasks (send and receive)
//!
use fo_fdc_comms::{
    battle_damage_assessment::BattleDamageAssessment,
    message_to_observer::MessageToObserver,
    readback::SolidReadback,
    request_for_fire::WarnOrder,
    shot_fire::{RoundsComplete, Shot, Splash},
};

/// Represents messages that can be received from the FDC
#[derive(Debug, PartialEq)]
pub(crate) enum FromFdcMessage {
    /// A readback for a RRF
    RequestForFireConfirm(WarnOrder),
    /// A Message to Observer, received after the FDC has received RFF
    MessageToObserver(MessageToObserver),
    /// Shot Call
    Shot(Shot),
    /// Splash Call
    Splash(Splash),
    /// Rounds Complete Call
    RoundsComplete(RoundsComplete),
    /// Indicates a SolidReadback (A message was confirmed as readback correctly)
    /// Primarily used for the readback of a MTO
    SolidReadback(SolidReadback),
}

/// Represents messages that can be sent to the FDC
#[derive(Debug, PartialEq)]
pub(crate) enum ToFdcMessage {
    /// The Request for Fire (RFF), the first message a FO will send the FDC to begin a fire mission
    RequestForFire(WarnOrder),
    /// Readback for an MTO
    MessageToObserverConfirm(MessageToObserver),
    /// Shot Readback
    ShotConfirm(Shot),
    /// Splash Readback
    SplashConfirm(Splash),
    /// RoundsComplete Readback
    RoundsCompleteConfirm(RoundsComplete),
    /// The Battle Damage Assessment (BDA), the last message a FO will send the FDC to end a fire mission
    BattleDamageAssessment(BattleDamageAssessment),
    /// Indicates a SolidReadback (A message was confirmed as readback correctly)
    /// Primarily used for the readback of a RRF or BDA
    SolidReadback(SolidReadback),
}
