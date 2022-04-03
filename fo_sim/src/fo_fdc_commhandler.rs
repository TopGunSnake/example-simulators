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

/// Provides a reader/writer loop, handling writes from the
pub(crate) async fn fo_fdc_commhandler_loop(
    to_fdc: UnboundedReceiver<FoFdcMessage>,
    from_fdc: UnboundedSender<FoFdcMessage>,
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
    from_fdc_sender: UnboundedSender<FoFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    let mut buffer = Vec::with_capacity(1024);

    while let Ok(_bytes_read) = socket.recv(&mut buffer).await {
        let value: FoFdcMessage = serde_json::from_slice(&buffer)?;

        from_fdc_sender.send(value)?;
    }
    Ok(())
}

async fn send_loop(
    mut to_fdc_receiver: UnboundedReceiver<FoFdcMessage>,
    socket: Arc<UdpSocket>,
) -> Result<()> {
    while let Some(message_to_fdc) = to_fdc_receiver.recv().await {
        let bytes = match message_to_fdc {
            FoFdcMessage::RequestForFire(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::MessageToObserverConfirm(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::ShotConfirm(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::SplashConfirm(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::RoundsCompleteConfirm(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::BattleDamageAssessment(msg) => serde_json::to_vec(&msg),
            FoFdcMessage::SolidReadback(msg) => serde_json::to_vec(&msg),
            _ => panic!(
                "Unsupported message was sent for transmission to the FDC: {:?}",
                message_to_fdc
            ),
        }?;

        socket.send(&bytes).await?;
    }

    Ok(())
}
