//! Provides the functions and enums for maintaining the FO state machine.
//!
//! The FO State Machine uses the top level [`FoState`] for representing the state of the FO.
//! The workhorse of this module, and the intended component for use is the [`state_machine_loop`],
//! which provides an `async` function for use in a runtime.
use anyhow::Result;
use fo_fdc_comms::{
    readback::SolidReadback,
    shot_fire::{Shot, Splash},
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{debug, info, info_span, instrument, trace, warn};

use crate::fo_fdc_commhandler::{FromFdcMessage, ToFdcMessage};

/// Representation of the top-level state of a Forward Observer
///
/// A FO is either offline (with no FDC to talk to), or connected to an FDC.
#[derive(Debug, PartialEq)]
pub(crate) enum FoState {
    /// State representing when the FO is not attached to an FDC.
    Offline,
    /// State representing when the FO is attached to an FDC.
    Connected {
        /// The callsign of the FDC that the FO is connected to.
        attached_fdc: String,
        /// The current state of the FO while connected.
        state: ConnectedState,
    },
}

impl FoState {
    /// Returns `true` if the fo state is [`Connected`].
    ///
    /// [`Connected`]: FoState::Connected
    #[must_use]
    pub(crate) fn is_connected(&self) -> bool {
        matches!(self, Self::Connected { .. })
    }

    /// Returns `true` if the fo state is [`Connected`] and [`Requesting`].
    ///
    /// [`Connected`]: FoState::Connected
    /// [`Requesting`]: ConnectedState::Requesting
    #[must_use]
    pub(crate) fn is_requesting(&self) -> bool {
        matches!(
            self,
            Self::Connected {
                state: ConnectedState::Requesting,
                ..
            }
        )
    }

    /// Returns `true` if the fo state is [`Connected`] and [`Observing`].
    ///
    /// [`Connected`]: FoState::Connected
    /// [`Observing`]: ConnectedState::Observing
    #[must_use]
    pub(crate) fn is_observing(&self) -> bool {
        matches!(
            self,
            Self::Connected {
                state: ConnectedState::Observing,
                ..
            }
        )
    }

    /// Tries to change the internal [`ConnectedState`] to [`Observing`].
    ///
    /// [`Observing`]: ConnectedState::Observing
    pub(crate) fn try_to_observing(self) -> Option<Self> {
        if let Self::Connected {
            attached_fdc,
            state,
        } = self
        {
            Some(Self::Connected {
                attached_fdc,
                state: ConnectedState::Observing,
            })
        } else {
            None
        }
    }
}

/// Representation of the Connectedstate of an FO.
///
/// While an FO is connected, it is either in standby (No request), requesting fires,
/// observing fires, or reporting a battle assessment
#[derive(Debug, PartialEq)]
pub(crate) enum ConnectedState {
    /// State representing when the FO is standing by, before requesting fires.
    Standby,
    /// State representing when the FO is requesting fires.
    Requesting,
    /// State representing when the FO is observing fires after receiving a MTO.
    Observing,
    /// State representing when the FO has finished observing, and is reporting a BDA back to the FDC.
    Reporting,
}

impl ConnectedState {
    /// Changes the [`ConnectedState`] to [`Observing`].
    ///
    /// [`Observing`]: ConnectedState::Observing
    pub(crate) fn into_observing(self) -> Self {
        Self::Observing
    }
}

/// Asynchronous executor loop for managing the state machine.
///
/// All state manipulation happens within the context of this function.
/// In the event a message is received that is not expected, the state machine will not change state,
/// but will emit a [`tracing::warn!`] event.
///
/// # Arguments
///
/// * `message_queue` - The receive side of a channel for processing the state machine with messages from the FDC.
/// * `to_fdc` - The send side of a channel where messages to send to the FDC are sent by the state machine loop.
#[instrument]
pub(crate) async fn state_machine_loop(
    mut message_queue: UnboundedReceiver<FromFdcMessage>,
    to_fdc: UnboundedSender<ToFdcMessage>,
) -> Result<()> {
    let mut state = FoState::Offline;
    let message_process_span = info_span!("message_process");

    // Receive messages from the queue until the senders close.
    // Once the senders close, this expression will return `None`.
    while let Some(message) = message_queue.recv().await {
        trace!("Received message: {:?}", message);
        let _enter = message_process_span.enter();

        match message {
            // Request for Fire Confirmation
            FromFdcMessage::RequestForFireConfirm(rff_readback) if state.is_requesting() => {
                info!("Received a readback for Request for Fire. Evaluating...");
                debug!("RFF Readback: {:?}", rff_readback);
                //TODO: Proccess any errors
                info!("Readback confirmed, sending SolidReadback...");
                to_fdc.send(ToFdcMessage::SolidReadback(SolidReadback))?
            }
            // MTO Received while Requesting a Fire Mission
            FromFdcMessage::MessageToObserver(mto) if state.is_requesting() => {
                info!("Received the MTO, reading back to FDC");
                to_fdc.send(ToFdcMessage::MessageToObserverConfirm(mto))?;
            }
            // Solid Readback received while requesting a Fire Mission
            FromFdcMessage::SolidReadback(_) if state.is_requesting() => {
                info!("Received a solid readback message. Transitioning to observing.");
                state = state
                    .try_to_observing()
                    .expect("state was invalid for conversion");
            }

            // Shot was received while Observing a Fire Mission
            FromFdcMessage::Shot(_msg) if state.is_observing() => {
                info!("Received a Shot message, echoing...");
                to_fdc.send(ToFdcMessage::ShotConfirm(Shot))?
            }
            // Splash was received while Observing a Fire Mission
            FromFdcMessage::Splash(_msg) if state.is_observing() => {
                info!("Received a Splash message, echoing...");
                to_fdc.send(ToFdcMessage::SplashConfirm(Splash))?
                //TODO: Start BDA send behavior
            }

            // UNEXPECTED MESSAGES
            FromFdcMessage::RequestForFireConfirm(rff_readback) => {
                warn!(
                    "Received a readback for RFF when in a state that does not expect an RFF: {:?}",
                    rff_readback
                );
            }
            FromFdcMessage::MessageToObserver(mto) => {
                warn!(
                    "Received an MTO when in a state that does not expect an MTO: {:?}",
                    mto
                );
            }
            FromFdcMessage::SolidReadback(msg) => {
                warn!("Received a solid readback when in a state that does not expect a solid readback: {:?}", msg);
            }
            FromFdcMessage::Shot(msg) => {
                warn!(
                    "Received a Shot when in a state that does not expect a Shot: {:?}",
                    msg
                );
            }
            FromFdcMessage::Splash(msg) => {
                warn!(
                    "Received a Splash when in a state that does not expect a Splash: {:?}",
                    msg
                );
            }
        }
    }
    Ok(())
}
