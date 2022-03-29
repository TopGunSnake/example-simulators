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

pub struct FireRequester {
    /// The socket the `FireRequester` will send the request from.
    socket: UdpSocket,
}

impl FireRequester {
    pub async fn new(fdc_address: SocketAddrV4) -> Self {
        let ip = Ipv4Addr::from_str("127.0.0.1").unwrap();
        let port = 8080;
        let addr = SocketAddrV4::new(ip, port);
        let sender_socket = UdpSocket::bind(addr)
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
