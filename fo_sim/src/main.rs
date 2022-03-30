use anyhow::Result;
use fo_fdc_comms::message_to_observer::MessageToObserver;
use serde_json::Value;
use tokio::{net::UdpSocket, signal};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, instrument, trace};

#[non_exhaustive]
#[derive(Debug)]
enum IncomingMessages {
    MessageToObserver(MessageToObserver),
}

#[derive(Debug)]
struct FoSimParams {
    callsign: String,
}

#[derive(Debug)]
struct FoSim {
    socket: UdpSocket,
    shutdown_token: CancellationToken,
}

impl FoSim {
    #[instrument]
    fn new(shutdown_token: CancellationToken, socket: UdpSocket) -> Self {
        Self {
            shutdown_token,
            socket,
        }
    }

    #[instrument]
    async fn run(&self) -> Result<()> {
        loop {
            trace!("Looping...");
            let msg = tokio::select! {
                _ = self.shutdown_token.cancelled() => break,
                result = self.recv_message() => match result {
                    Ok(msg) => Self::process_message(&msg).await?,
                    Err(err) => return Err(err),
                }
            };
            match msg {
                IncomingMessages::MessageToObserver(mto) => self.handle_mto(mto).await,
            }
        }
        info!("Simulator stopped");
        Ok(())
    }

    #[instrument]
    async fn recv_message(&self) -> Result<Vec<u8>> {
        let mut buffer = vec![0u8; 20];
        debug!("Buffer size: {} bytes", buffer.len());

        let bytes_read = self.socket.recv(&mut buffer).await?;
        debug!("{} bytes received", bytes_read);

        buffer.truncate(bytes_read);

        Ok(buffer)
    }

    /// Helper function to convert a received json message into one of the formats expected.
    #[instrument]
    async fn process_message(msg: &[u8]) -> Result<IncomingMessages> {
        let v: Value = serde_json::from_slice(msg)?;
        info!("Received message: {:#?}", v);

        //TODO: Write logic to conditionally convert to Incoming message types
        let message: MessageToObserver = serde_json::from_value(v)?;
        Ok(IncomingMessages::MessageToObserver(message))
    }

    /// Handle a received Message To Observer
    #[instrument]
    async fn handle_mto(&self, mto: MessageToObserver) {
        info!("Received MTO: {:?}", mto);
        //TODO: Handle!
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting FO Simulator");
    let shutdown_token = CancellationToken::new();
    let child_token = shutdown_token.child_token();
    info!("Setting up UDP Socket to FDC");
    let fdc_socket = UdpSocket::bind("127.0.0.1:7070").await?;

    fdc_socket.connect("127.0.0.1:8080").await?;

    let join_handle = tokio::spawn(async move {
        let fo_sim = FoSim::new(child_token, fdc_socket);
        fo_sim.run().await
    });
    tokio::select! {
        _ = signal::ctrl_c() => {
            shutdown_token.cancel();
        },
        result = join_handle => {}
    }

    info!("FO Simulator shutdown");
    Ok(())
}

#[cfg(test)]
mod tests {
    use fo_fdc_comms::{message_to_observer::TargetNumber, Ammunition};

    use super::*;

    #[tokio::test]
    async fn test_message_handling() {
        let mto = MessageToObserver {
            src: "HK4A1T".to_string(),
            receiver: "FD1I5B".to_string(),
            target_number: TargetNumber::new("AN2001").unwrap(),
            ammunition: Ammunition::HighExplosive,
            rounds: 4,
        };

        let bytes = serde_json::to_vec(&mto).unwrap();

        let message = FoSim::process_message(&bytes).await.unwrap();

        if let IncomingMessages::MessageToObserver(msg) = message {
            assert_eq!(msg, mto);
        } else {
            panic!("Wrong type was unpacked: {:?}", message);
        }
    }
}
