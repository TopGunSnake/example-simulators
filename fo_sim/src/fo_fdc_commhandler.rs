use fo_fdc_comms::{
    message_to_observer::MessageToObserver,
    shot_fire::{Shot, Splash},
};

#[derive(Debug, PartialEq)]
pub(crate) enum FoMessage {
    MessageToObserver(MessageToObserver),
    Shot(Shot),
    Splash(Splash),
}
