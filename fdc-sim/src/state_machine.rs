//! Provides the functions and enums for maintaining the FDC state machine.
//!
//! The FDC State Machine uses the top level [`FdcState`] for representing the state of the FDC.
//! The workhorse of this module, and the intended component for use is the [`state_machine_loop`],
//! which provides an `async` function for use in a runtime.
use anyhow::Result;
use fo_fdc_comms::{
    battle_damage_assessment::BattleDamageAssessment,
    message_to_observer::{MessageToObserver, TargetNumber},
    readback::SolidReadback,
    request_for_fire::{MissionType, TargetDescription, TargetLocation, WarnOrder},
    shot_fire::{RoundsComplete, Shot, Splash},
    Ammunition, FoFdcMessage,
};
use tokio::{
    sync::mpsc::{error::TryRecvError, UnboundedReceiver, UnboundedSender},
    task::yield_now,
};
use tracing::{debug, error, info, info_span, trace, warn};

#[derive(Debug, Clone, Copy)]
enum FdcState {
    Offline,
    Online { state: OnlineState },
}

#[derive(Debug, Clone, Copy)]
enum OnlineState {
    Waiting,
    Firing,
}

/// Asynchronous executor loop for managing the state machine.
///
/// All state manipulation happens within the context of this function.
/// In the event a message is received that is not expected, the state machine will not change state,
/// but will emit a [`tracing::warn!`] event.
///
/// # Arguments
///
/// * `from_fo_rx` - The receive side of a channel for processing the state machine with messages from the FO.
/// * `to_fo_tx` - The send side of a channel where messages to send to the FO are sent by the state machine loop.
pub(crate) async fn state_machine_loop(
    mut from_fo_rx: UnboundedReceiver<FoFdcMessage>,
    to_fo_tx: UnboundedSender<FoFdcMessage>,
) -> Result<()> {
    let mut state = FdcState::Online {
        state: OnlineState::Waiting,
    };
    let message_process_span = info_span!("message_process");
    let state_machine_run_span = info_span!("state_run");
    info!("Starting state machine...");
    loop {
        trace!("Looping...");
        debug!("State is {:?}", state);

        let message = match from_fo_rx.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => {
                info!("transmitter disconnected");
                break;
            }
        };

        tokio::task::yield_now().await;

        trace!("Checking for messages");
        if let Some(message) = message {
            debug!("Received message: {:?}", message);
            let _enter = message_process_span.enter();
            trace!("Entering message processor");
            match (message, state) {
                // Request for Fire received while online
                (
                    FoFdcMessage::RequestForFire(rff),
                    FdcState::Online {
                        state: OnlineState::Waiting,
                    },
                ) => {
                    info!("Received RfF, handling...");
                    state = FdcState::Online {
                        state: OnlineState::Firing,
                    };
                    to_fo_tx.send(FoFdcMessage::RequestForFireConfirm(rff))?;
                }
                // Solid Readback received for our RFF Confirmation
                (
                    FoFdcMessage::SolidReadback(SolidReadback::RequestForFire),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Solid Readback for RFF, proceeding to fire.");
                    let mto = MessageToObserver {
                        src: "FDC".to_string(),
                        receiver: "FO".to_string(),
                        target_number: TargetNumber::new("AN2001").unwrap(),
                        ammunition: Ammunition::HighExplosive,
                        rounds: 4,
                    };
                    debug!("MTO: {:?}", mto);
                    to_fo_tx.send(FoFdcMessage::MessageToObserver(mto))?;
                }
                // MTO Readback received
                (
                    FoFdcMessage::MessageToObserverConfirm(mto_readback),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Received readback for MTO");
                    //TODO: Verify MTO
                    to_fo_tx.send(FoFdcMessage::SolidReadback(
                        SolidReadback::MessageToObserver,
                    ))?;
                    tokio::time::sleep(tokio::time::Duration::from_secs(13)).await;

                    to_fo_tx.send(FoFdcMessage::Shot(Shot {}))?;
                    for shot_number in 0..3 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        to_fo_tx.send(FoFdcMessage::Shot(Shot {}))?;
                    }

                    tokio::time::sleep(tokio::time::Duration::from_secs(13)).await;
                    to_fo_tx.send(FoFdcMessage::Splash(Splash {}))?;

                    tokio::time::sleep(tokio::time::Duration::from_secs(4)).await;
                    to_fo_tx.send(FoFdcMessage::RoundsComplete(RoundsComplete {}))?;
                }

                // Handle Shot Readbacks
                (
                    FoFdcMessage::ShotConfirm(shot_readback),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Received readback for shot");
                    //TODO: Verify Readback
                    to_fo_tx.send(FoFdcMessage::SolidReadback(SolidReadback::Shot))?;
                }
                // Handle Splash Readbacks
                (
                    FoFdcMessage::SplashConfirm(splash_readback),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Received readback for splash");
                    //TODO: Verify Readback
                    to_fo_tx.send(FoFdcMessage::SolidReadback(SolidReadback::Splash))?;
                }
                // Handle Rounds Complete Readback
                (
                    FoFdcMessage::RoundsCompleteConfirm(rounds_complete_readback),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Received readback for rounds complete");
                    //TODO: Verify Readback
                    to_fo_tx.send(FoFdcMessage::SolidReadback(SolidReadback::RoundsComplete))?;
                    // Now we wait for BDA
                }
                // Handle BDA
                (
                    FoFdcMessage::BattleDamageAssessment(bda),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("Received BDA");
                    to_fo_tx.send(FoFdcMessage::BattleDamageAssessmentConfirm(bda))?;
                }
                // Handle Solid Readback for BDA
                (
                    FoFdcMessage::SolidReadback(SolidReadback::BattleDamageAssessment),
                    FdcState::Online {
                        state: OnlineState::Firing,
                    },
                ) => {
                    info!("BDA Readback confirmed. Transitioning back to waiting.");
                    state = FdcState::Online {
                        state: OnlineState::Waiting,
                    }
                }

                // Unexpected messages
                (_, _) => {
                    warn!("Invalid message received, or received in invalid state");
                }
            }
        }

        let _state_run_enter = state_machine_run_span.enter();
        trace!("Entering the state runner");
        match state {
            FdcState::Offline => {
                state = FdcState::Online {
                    state: OnlineState::Waiting,
                }
            }
            FdcState::Online { .. } => (),
        }
    }
    Ok(())
}
