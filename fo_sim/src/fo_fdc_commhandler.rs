use fo_fdc_comms::{
    battle_damage_assessment::BattleDamageAssessment,
    message_to_observer::MessageToObserver,
    readback::SolidReadback,
    request_for_fire::WarnOrder,
    shot_fire::{Shot, Splash},
};

#[derive(Debug, PartialEq)]
pub(crate) enum FromFdcMessage {
    RequestForFireConfirm(WarnOrder),
    MessageToObserver(MessageToObserver),
    Shot(Shot),
    Splash(Splash),
    SolidReadback(SolidReadback),
}

#[derive(Debug, PartialEq)]
pub(crate) enum ToFdcMessage {
    RequestForFire(WarnOrder),
    MessageToObserverConfirm(MessageToObserver),
    ShotConfirm(Shot),
    SplashConfirm(Splash),
    BattleDamageAssessment(BattleDamageAssessment),
    SolidReadback(SolidReadback),
}
