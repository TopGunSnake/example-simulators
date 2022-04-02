//! Contains the message types that the FO sim can send/receive, as well as the communication tasks (send and receive)
//!
use std::sync::Arc;

use anyhow::Result;
use fo_fdc_comms::{
    battle_damage_assessment::BattleDamageAssessment,
    message_to_observer::MessageToObserver,
    readback::SolidReadback,
    request_for_fire::WarnOrder,
    shot_fire::{RoundsComplete, Shot, Splash},
};
use serde_json::Value;
use tokio::{
    join,
    net::UdpSocket,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
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

/// Provides a reader/writer loop, handling writes from the
pub(crate) async fn fo_fdc_commhandler_loop(
    to_fdc: UnboundedReceiver<ToFdcMessage>,
    from_fdc: UnboundedSender<FromFdcMessage>,
) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    socket.connect("127.0.0.1:8081").await?;

    // Spin off listener thread
    let socket = Arc::new(socket);
    let recv_handle = {
        let socket = Arc::clone(&socket);
        tokio::spawn(async move { recv_loop(from_fdc, socket).await })
    };

    // Spin off writer thread
    let send_handle = tokio::spawn(async move { send_loop(to_fdc, socket).await });

    let _ = join!(recv_handle, send_handle);

    Ok(())
}

async fn recv_loop(
    from_fdc_sender: UnboundedSender<FromFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    let mut buffer = Vec::with_capacity(1024);

    while let Ok(_bytes_read) = socket.recv(&mut buffer).await {
        let value: Value = serde_json::from_slice(&buffer)?;

        let result = match value {
            // Incoming Request for Fire Confirmation
            Value::Object(map) if map.contains_key("mission_type") => Ok(
                FromFdcMessage::RequestForFireConfirm(serde_json::from_slice(&buffer)?),
            ),
            // Incoming Message to Observer
            Value::Object(map) if map.contains_key("") => Ok(FromFdcMessage::MessageToObserver(
                serde_json::from_slice(&buffer)?,
            )),
            // Incoming Shot
            Value::Object(map) if map.contains_key("") => {
                Ok(FromFdcMessage::Shot(serde_json::from_slice(&buffer)?))
            }
            // Incoming Splash
            Value::Object(map) if map.contains_key("") => {
                Ok(FromFdcMessage::Splash(serde_json::from_slice(&buffer)?))
            }
            // Incoming Rounds Complete
            Value::Object(map) if map.contains_key("") => Ok(FromFdcMessage::RoundsComplete(
                serde_json::from_slice(&buffer)?,
            )),
            // Incoming Solid Readback
            Value::Object(map) if map.contains_key("") => Ok(FromFdcMessage::SolidReadback(
                serde_json::from_slice(&buffer)?,
            )),
            _ => Err(std::io::Error::from(std::io::ErrorKind::InvalidData)),
        }?;

        from_fdc_sender.send(result)?;
    }
    Ok(())
}

async fn send_loop(
    mut to_fdc_receiver: UnboundedReceiver<ToFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    while let Some(message_to_fdc) = to_fdc_receiver.recv().await {
        let bytes = match message_to_fdc {
            ToFdcMessage::RequestForFire(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::MessageToObserverConfirm(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::ShotConfirm(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::SplashConfirm(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::RoundsCompleteConfirm(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::BattleDamageAssessment(msg) => serde_json::to_vec(&msg)?,
            ToFdcMessage::SolidReadback(msg) => serde_json::to_vec(&msg)?,
        };

        socket.send(&bytes).await?;
    }

    Ok(())
}
