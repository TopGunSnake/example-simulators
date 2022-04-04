//! Contains the message types that the FO sim can send/receive, as well as the communication tasks (send and receive)
//!
use std::sync::Arc;

use anyhow::Result;
use fo_fdc_comms::FoFdcMessage;
use tokio::{
    join,
    net::UdpSocket,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tracing::debug;

/// Provides a reader/writer loop, handling writes from the
pub(crate) async fn fo_fdc_commhandler_loop(
    to_fdc: UnboundedReceiver<FoFdcMessage>,
    from_fdc: UnboundedSender<FoFdcMessage>,
) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:49152").await?;

    socket.connect("127.0.0.1:49153").await?;
    debug!("Bound to {}", socket.local_addr()?);

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
    from_fdc_sender: UnboundedSender<FoFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    let mut buffer = vec![0; 24 * 1024];

    while let Ok(bytes_read) = socket.recv(&mut buffer).await {
        let value: FoFdcMessage = serde_json::from_slice(&buffer[..bytes_read])?;
        debug!("Received {:?}", value);
        from_fdc_sender.send(value)?;
    }
    Ok(())
}

async fn send_loop(
    mut to_fdc_receiver: UnboundedReceiver<FoFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    while let Some(message_to_fdc) = to_fdc_receiver.recv().await {
        debug!("Sending {:?}", message_to_fdc);
        let (message_json, bytes) = match message_to_fdc {
            FoFdcMessage::RequestForFire(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            FoFdcMessage::MessageToObserverConfirm(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),

            FoFdcMessage::ShotConfirm(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            FoFdcMessage::SplashConfirm(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            FoFdcMessage::RoundsCompleteConfirm(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            FoFdcMessage::BattleDamageAssessment(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            FoFdcMessage::SolidReadback(..) => (
                serde_json::to_string_pretty(&message_to_fdc)?,
                serde_json::to_vec(&message_to_fdc)?,
            ),
            _ => panic!(
                "Unsupported message was sent for transmission to the FDC: {:?}",
                message_to_fdc
            ),
        };
        debug!("Sending: {}", message_json);
        let bytes_sent = socket.send(&bytes).await?;
        debug!("Sent {} bytes of {} ", bytes_sent, bytes.len());
    }

    Ok(())
}
