#![forbid(unused_imports)]

use std::{
    net::{Ipv4Addr, SocketAddrV4},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;

pub mod message_to_observer;
pub mod request_for_fire;

/// Ammunition types
#[non_exhaustive]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ammunition {
    HighExplosive,
}

/// A Fire Requester, providing a wrapper around the JSON over UDP interface
pub struct FireRequester {
    /// The socket the `FireRequester` will send the request from.
    socket: UdpSocket,
}

impl FireRequester {
    /// Initializes a `FireRequester` that talks to the specified FDC Address
    pub async fn new(fdc_address: SocketAddrV4) -> Self {
        let sender_socket = UdpSocket::bind("127.0.0.1:8080")
            .await
            .expect("failed to bind to address");
        sender_socket
            .connect(fdc_address)
            .await
            .expect("failed to connect to target address");

        Self {
            socket: sender_socket,
        }
    }

    /// Sends a RRF on this `FireRequester`'s connected socket.
    pub async fn send_request_for_fire(
        &self,
        rrf: request_for_fire::WarnOrder,
    ) -> Result<(), String> {
        let bytes = serde_json::to_vec(&rrf).map_err(|err| err.to_string())?;
        let bytes_written = self
            .socket
            .send(&bytes)
            .await
            .map_err(|err| err.to_string())?;
        if bytes.len() != bytes_written {
            Err("The RRF may have only been partially sent".to_string())
        } else {
            Ok(())
        }
    }
}
