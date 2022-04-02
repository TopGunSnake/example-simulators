//! Provides the functions and enums for maintaining the FO state machine.
//!
//! The FO State Machine uses the top level [`FoState`] for representing the state of the FO.
//! The workhorse of this module, and the intended component for use is the [`state_machine_loop`],
//! which provides an `async` function for use in a runtime.
use anyhow::Result;
use fo_fdc_comms::{
    battle_damage_assessment::BattleDamageAssessment,
    readback::SolidReadback,
    shot_fire::{RoundsComplete, Shot, Splash},
    FoFdcMessage,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tracing::{debug, error, info, info_span, instrument, trace, warn};

/// Representation of the top-level state of a Forward Observer
///
/// A FO is either offline (with no FDC to talk to), or connected to an FDC.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum FoState {
    /// State representing when the FO is not attached to an FDC.
    Offline,
    /// State representing when the FO is attached to an FDC.
    Connected {
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
        if let Self::Connected { state } = self {
            Some(Self::Connected {
                state: ConnectedState::Observing,
            })
        } else {
            None
        }
    }

    /// Tries to change the internal [`ConnectedState`] to [`Reporting`]
    ///
    /// [`Reporting`]: ConnectedState::Reporting
    pub(crate) fn try_to_reporting(self) -> Option<Self> {
        if let Self::Connected { state } = self {
            Some(Self::Connected {
                state: ConnectedState::Reporting,
            })
        } else {
            None
        }
    }

    /// Tries to change the internal [`ConnectedState`] to [`Standby`]
    ///
    /// [`Standby`]: ConnectedState::Standby
    pub(crate) fn try_to_standby(self) -> Option<Self> {
        if let Self::Connected { state } = self {
            Some(Self::Connected {
                state: ConnectedState::Standby,
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
#[derive(Debug, Clone, Copy, PartialEq)]
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
    mut message_queue: UnboundedReceiver<FoFdcMessage>,
    to_fdc: UnboundedSender<FoFdcMessage>,
) -> Result<()> {
    let mut state = FoState::Offline;
    let message_process_span = info_span!("message_process");

    // Receive messages from the queue until the senders close.
    // Once the senders close, this expression will return `None`.
    while let Some(message) = message_queue.recv().await {
        trace!("Received message: {:?}", message);
        let _enter = message_process_span.enter();

        match (message, state) {
            // FDC Messages sent when the FO is requesting

            // Request for Fire Confirmation
            (
                FoFdcMessage::RequestForFireConfirm(rff_readback),
                FoState::Connected {
                    state: ConnectedState::Requesting,
                },
            ) => {
                info!("Received a readback for Request for Fire. Evaluating...");
                debug!("RFF Readback: {:?}", rff_readback);
                //TODO: Proccess any errors
                info!("Readback confirmed, sending SolidReadback...");
                to_fdc.send(FoFdcMessage::SolidReadback(SolidReadback::RequestForFire))?
            }
            // MTO Received while Requesting a Fire Mission
            (
                FoFdcMessage::MessageToObserver(mto),
                FoState::Connected {
                    state: ConnectedState::Requesting,
                },
            ) => {
                info!("Received the MTO, reading back to FDC");
                to_fdc.send(FoFdcMessage::MessageToObserverConfirm(mto))?;
            }
            // Solid Readback received while requesting a Fire Mission
            (
                FoFdcMessage::SolidReadback(SolidReadback::MessageToObserver),
                FoState::Connected {
                    state: ConnectedState::Requesting,
                },
            ) => {
                info!("Received a solid readback message. Transitioning to observing.");
                state = state
                    .try_to_observing()
                    .expect("state was invalid for conversion");
            }

            // FDC Messages sent when the FO is observing

            // Shot was received while Observing a Fire Mission
            (
                FoFdcMessage::Shot(_),
                FoState::Connected {
                    state: ConnectedState::Observing,
                },
            ) => {
                info!("Received a Shot message, echoing...");
                to_fdc.send(FoFdcMessage::ShotConfirm(Shot::default()))?
            }

            // Splash was received while Observing a Fire Mission
            (
                FoFdcMessage::Splash(_),
                FoState::Connected {
                    state: ConnectedState::Observing,
                },
            ) => {
                info!("Received a Splash message, echoing...");
                to_fdc.send(FoFdcMessage::SplashConfirm(Splash::default()))?
            }

            // RoundsComplete was received while Observing a Fire Mission
            (
                FoFdcMessage::RoundsComplete(_),
                FoState::Connected {
                    state: ConnectedState::Observing,
                },
            ) => {
                info!("Received a RoundsComplete message, echoing...");
                to_fdc.send(FoFdcMessage::RoundsCompleteConfirm(
                    RoundsComplete::default(),
                ))?
            }

            // Solid readback was received while observing
            (
                FoFdcMessage::SolidReadback(SolidReadback::Shot | SolidReadback::Splash),
                FoState::Connected {
                    state: ConnectedState::Observing,
                },
            ) => {
                info!("Received a solid readback message. Waiting.");
            }

            // Solid readback was received for Rounds Complete while observing
            (
                FoFdcMessage::SolidReadback(SolidReadback::RoundsComplete),
                FoState::Connected {
                    state: ConnectedState::Observing,
                },
            ) => {
                info!("Received a solid readback for rounds complete. Reporting a BDA");

                state = state
                    .try_to_reporting()
                    .expect("state was invalid for conversion");

                to_fdc.send(FoFdcMessage::BattleDamageAssessment(
                    BattleDamageAssessment {},
                ))?
            }

            // FDC Messages sent when the FO is reporting

            // Received BDA Readback
            (
                FoFdcMessage::BattleDamageAssessmentConfirm(bda_readback),
                FoState::Connected {
                    state: ConnectedState::Reporting,
                },
            ) => {
                info!("Received a readback for Battle Damage Assessment. Evaluating...");
                debug!("BDA Readback: {:?}", bda_readback);
                //TODO: Proccess any errors
                info!("Readback confirmed, sending SolidReadback...");
                to_fdc.send(FoFdcMessage::SolidReadback(
                    SolidReadback::BattleDamageAssessment,
                ))?;

                state = state
                    .try_to_standby()
                    .expect("state was invalid for conversion")
            }

            // UNEXPECTED MESSAGES
            (FoFdcMessage::RequestForFireConfirm(_), _)
            | (FoFdcMessage::MessageToObserver(_), _)
            | (FoFdcMessage::SolidReadback(_), _)
            | (FoFdcMessage::Shot(_), _)
            | (FoFdcMessage::Splash(_), _)
            | (FoFdcMessage::RoundsComplete(_), _)
            | (FoFdcMessage::BattleDamageAssessmentConfirm(_), _) => {
                warn!("Received a message when in a state that doesn't expect it",);
            }

            // Invalid messages, these messages are not expected, since we only send these.
            (FoFdcMessage::RequestForFire(_), _)
            | (FoFdcMessage::BattleDamageAssessment(_), _)
            | (FoFdcMessage::MessageToObserverConfirm(_), _)
            | (FoFdcMessage::RoundsCompleteConfirm(_), _)
            | (FoFdcMessage::ShotConfirm(_), _)
            | (FoFdcMessage::SplashConfirm(_), _) => {
                error!("Received a message intended for transmission from FO Sim only",)
            }
        }
    }
    Ok(())
}
