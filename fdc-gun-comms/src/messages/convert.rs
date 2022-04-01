//! Trait definitions for message conversions

use std::collections::HashMap;

use thiserror::Error;

use crate::FdcGunMessage;

use super::{
    status::{Ammunition, Status},
    CheckFire, ComplianceResponse, FireCommand, FireReport, StatusReply, StatusRequest,
};

#[derive(Error, Debug)]
#[error("Failed to convert")]
pub struct MessageConvertError;

impl From<StatusRequest> for FdcGunMessage {
    fn from(_: StatusRequest) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for StatusRequest {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<StatusReply> for FdcGunMessage {
    fn from(status_reply: StatusReply) -> Self {
        let status = status_reply.status as u8;
        let rounds = status_reply
            .rounds
            .into_iter()
            .flat_map(|(ammunition, count)| {
                let mut piece = Vec::with_capacity(1 + 4);
                piece.push(ammunition as u8);
                piece.extend(count.to_be_bytes());
                piece
            });

        let message = std::iter::once(status).chain(rounds).collect::<Vec<u8>>();

        Self {
            message_id: crate::FdcGunMessageId::StatusReply,
            message_contents: message,
        }
    }
}

impl TryFrom<FdcGunMessage> for StatusReply {
    type Error = MessageConvertError;

    fn try_from(message: FdcGunMessage) -> Result<Self, Self::Error> {
        if message.message_id != crate::FdcGunMessageId::StatusReply {
            Err(MessageConvertError)
        } else {
            // Unpack the first byte into a Status
            let status = Status::try_from(
                message
                    .message_contents
                    .get(0)
                    .ok_or(MessageConvertError)?
                    .to_owned(),
            )
            .map_err(|_| MessageConvertError)?;

            // Unpack each 5 bytes after into an Ammunition, Count pair, and collect
            // into a HashMap
            let rounds = message.message_contents[1..]
                .chunks_exact(1 + 5)
                .map(|key_value| {
                    let ammunition =
                        Ammunition::try_from(key_value[0]).expect("Ammunition is unknown");
                    let count = u32::from_be_bytes(
                        key_value[1..]
                            .try_into()
                            .expect("Chunk was incorrectly sized"),
                    );

                    (ammunition, count)
                })
                .collect::<HashMap<Ammunition, u32>>();
            Ok(Self { status, rounds })
        }
    }
}

impl From<FireReport> for FdcGunMessage {
    fn from(_: FireReport) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for FireReport {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<FireCommand> for FdcGunMessage {
    fn from(_: FireCommand) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for FireCommand {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<CheckFire> for FdcGunMessage {
    fn from(_: CheckFire) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for CheckFire {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl From<ComplianceResponse> for FdcGunMessage {
    fn from(_: ComplianceResponse) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for ComplianceResponse {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        messages::{
            CheckFire, ComplianceResponse, FireCommand, FireReport, StatusReply, StatusRequest,
        },
        FdcGunMessage,
    };

    #[test]
    fn test_conversion_properties() {
        // test_type(CheckFire::default());
        // test_type(ComplianceResponse::default());
        // test_type(FireCommand::default());
        // test_type(FireReport::default());
        test_type(StatusReply::default());
        // test_type(StatusRequest::default());
    }

    fn test_type<T>(input: T)
    where
        T: Into<FdcGunMessage> + std::convert::TryFrom<FdcGunMessage> + std::fmt::Debug,
        <T as std::convert::TryFrom<FdcGunMessage>>::Error: std::fmt::Debug,
    {
        let message: FdcGunMessage = input.into();
        let output: T = message.try_into().unwrap();
        // assert_eq!(input, output);
    }
}
