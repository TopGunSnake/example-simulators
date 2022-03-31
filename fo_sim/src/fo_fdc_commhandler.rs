use fo_fdc_comms::message_to_observer::MessageToObserver;

#[derive(Debug, PartialEq)]
pub(crate) enum FoMessage {
    MessageToObserver(MessageToObserver),
}
