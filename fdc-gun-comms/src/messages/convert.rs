//! Trait definitions for message conversions

use std::collections::HashMap;

use thiserror::Error;

use crate::FdcGunMessage;

use super::{
    status::{Ammunition, Status},
    CheckFire, ComplianceResponse, FireCommand, FireReport, StatusReply, StatusRequest,
};

///
#[derive(Error, Debug)]
pub enum MessageConvertError {
    /// Indicates that the message ID from the data does not match the type the data is being converted into
    #[error("The message ID does not match the type")]
    IdMismatch,
    /// Indicates that the message data is missing parts needed to construct the strongly-typed message
    #[error("The message is missing data needed for conversion")]
    MissingData,
    /// Indicates that the data in the message is invalid
    #[error("The message contains invalid data")]
    InvalidData,
}

impl From<StatusRequest> for FdcGunMessage {
    fn from(_status_request: StatusRequest) -> Self {
        Self {
            message_id: crate::FdcGunMessageId::StatusRequest,
            message_contents: Vec::with_capacity(0),
        }
    }
}

impl TryFrom<FdcGunMessage> for StatusRequest {
    type Error = MessageConvertError;

    fn try_from(message: FdcGunMessage) -> Result<Self, Self::Error> {
        if message.message_id != crate::FdcGunMessageId::StatusRequest {
            Err(MessageConvertError::IdMismatch)
        } else {
            Ok(Self {})
        }
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
            Err(MessageConvertError::IdMismatch)
        } else {
            // Unpack the first byte into a Status
            let status = Status::try_from(
                message
                    .message_contents
                    .get(0)
                    .ok_or(MessageConvertError::MissingData)?
                    .to_owned(),
            )
            .map_err(|_| MessageConvertError::InvalidData)?;

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
    fn from(compliance_response: ComplianceResponse) -> Self {
        let message_contents = vec![compliance_response.compliance as u8];
        Self {
            message_id: crate::FdcGunMessageId::ComplianceResponse,
            message_contents,
        }
    }
}

impl TryFrom<FdcGunMessage> for ComplianceResponse {
    type Error = MessageConvertError;

    fn try_from(message: FdcGunMessage) -> Result<Self, Self::Error> {
        if message.message_id != crate::FdcGunMessageId::ComplianceResponse {
            Err(MessageConvertError::IdMismatch)
        } else {
            let compliance = message
                .message_contents
                .get(0)
                .ok_or(MessageConvertError::MissingData)?
                .to_owned()
                .try_into()
                .map_err(|_| MessageConvertError::InvalidData)?;

            Ok(Self { compliance })
        }
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
        test_type(ComplianceResponse::default());
        // test_type(FireCommand::default());
        // test_type(FireReport::default());
        test_type(StatusReply::default());
        test_type(StatusRequest::default());
    }

    fn test_type<T>(input: T)
    where
        T: Into<FdcGunMessage>
            + std::convert::TryFrom<FdcGunMessage>
            + std::fmt::Debug
            + std::cmp::PartialEq
            + Clone,
        <T as std::convert::TryFrom<FdcGunMessage>>::Error: std::fmt::Debug,
    {
        let message: FdcGunMessage = input.clone().into();
        let output: T = message.try_into().unwrap();
        assert_eq!(input, output);
    }
}
