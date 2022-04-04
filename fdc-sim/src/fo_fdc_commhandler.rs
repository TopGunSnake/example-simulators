//! Contains the message types that the FDC sim can send/receive, as well as the communication tasks (send and receive)
//!
use std::sync::Arc;

use anyhow::Result;
use fo_fdc_comms::FoFdcMessage;
use tokio::{
    net::UdpSocket,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
    try_join,
};
use tracing::{debug, trace};

/// Provides a reader/writer loop, sending data from `to_fo_rx` to the FO over UDP,
/// and placing data received from the FO in the `from_fo_tx`.
pub(crate) async fn fo_fdc_commhandler_loop(
    to_fo_rx: UnboundedReceiver<FoFdcMessage>,
    from_fo_tx: UnboundedSender<FoFdcMessage>,
) -> Result<()> {
    let socket = UdpSocket::bind("127.0.0.1:49153").await?;
    socket.connect("127.0.0.1:49152").await?;
    debug!("Bound to {}", socket.local_addr()?);

    // Spin off listener thread
    let socket = Arc::new(socket);
    let recv_handle = {
        let socket = Arc::clone(&socket);
        tokio::task::Builder::new()
            .name("receive loop")
            .spawn(async move { recv_loop(from_fo_tx, socket).await })
    };

    // Spin off writer thread
    let send_handle = tokio::task::Builder::new()
        .name("send loop")
        .spawn(async move { send_loop(to_fo_rx, socket).await });

    let (left, right) = try_join!(recv_handle, send_handle)?;
    left?;
    right?;

    Ok(())
}

/// Receive loop, for handling receipt over UDP
async fn recv_loop(
    from_fo_tx: UnboundedSender<FoFdcMessage>,
    fo_socket: Arc<UdpSocket>,
) -> Result<()> {
    let mut buffer = vec![0; 24 * 1024];
    trace!("Started the recv loop");
    loop {
        match fo_socket.recv(&mut buffer).await {
            Ok(bytes_read) => {
                let value: FoFdcMessage = serde_json::from_slice(&buffer[..bytes_read])?;
                debug!("Received {:?}", value);
                from_fo_tx.send(value)?;
            }
            Err(err) => return Err(err.into()),
        };
    }
    // while let Ok(_bytes_read) = fo_socket.recv(&mut buffer).await {}
    // trace!("Leaving the recv loop");
    // Ok(())
}

/// Send loop, for sending data over UDP
async fn send_loop(
    mut to_fo_rx: UnboundedReceiver<FoFdcMessage>,
    fo_socket: Arc<UdpSocket>,
) -> Result<()> {
    trace!("Started the send loop");
    while let Some(message_to_fo) = to_fo_rx.recv().await {
        debug!("Sending {:?}", message_to_fo);
        let bytes = match message_to_fo {
            FoFdcMessage::RequestForFireConfirm(..)
            | FoFdcMessage::MessageToObserver(..)
            | FoFdcMessage::Shot(..)
            | FoFdcMessage::Splash(..)
            | FoFdcMessage::RoundsComplete(..)
            | FoFdcMessage::BattleDamageAssessmentConfirm(..)
            | FoFdcMessage::SolidReadback(..) => serde_json::to_vec(&message_to_fo),
            _ => panic!(
                "Unsupported message was sent for transmission to the FO: {:?}",
                message_to_fo
            ),
        }?;

        let bytes_sent = fo_socket.send(&bytes).await?;
        debug!("Sent {} bytes of {} ", bytes_sent, bytes.len());
    }
    trace!("Leaving the send loop");
    Ok(())
}
