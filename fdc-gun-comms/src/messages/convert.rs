//! Trait definitions for message conversions

use thiserror::Error;

use crate::FdcGunMessage;

use super::{CheckFire, ComplianceResponse, FireCommand, FireReport, StatusReply, StatusRequest};

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
    fn from(_: StatusReply) -> Self {
        todo!()
    }
}

impl TryFrom<FdcGunMessage> for StatusReply {
    type Error = MessageConvertError;

    fn try_from(value: FdcGunMessage) -> Result<Self, Self::Error> {
        todo!()
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
        test_type(CheckFire::default());
        test_type(ComplianceResponse::default());
        test_type(FireCommand::default());
        test_type(FireReport::default());
        test_type(StatusReply::default());
        test_type(StatusRequest::default());
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
