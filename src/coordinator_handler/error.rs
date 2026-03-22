use serde::{Deserialize, Deserializer};
use strum::EnumIs;

#[derive(Debug, Clone, Copy, PartialEq, EnumIs)]
pub enum ErrorCode {
    Success,
    InternalServerError,

    ProverStatsAPIParameterInvalidNo,
    ProverStatsAPIProverTaskFailure,
    ProverStatsAPIProverTotalRewardFailure,

    CoordinatorParameterInvalidNo,
    CoordinatorGetTaskFailure,
    CoordinatorHandleZkProofFailure,
    CoordinatorEmptyProofData,

    JWTCommonErr,
    JWTTokenExpired,

    Undefined(i32),
}

impl ErrorCode {
    fn from_i32(v: i32) -> Self {
        match v {
            0 => ErrorCode::Success,
            500 => ErrorCode::InternalServerError,
            10001 => ErrorCode::ProverStatsAPIParameterInvalidNo,
            10002 => ErrorCode::ProverStatsAPIProverTaskFailure,
            10003 => ErrorCode::ProverStatsAPIProverTotalRewardFailure,
            20001 => ErrorCode::CoordinatorParameterInvalidNo,
            20002 => ErrorCode::CoordinatorGetTaskFailure,
            20003 => ErrorCode::CoordinatorHandleZkProofFailure,
            20004 => ErrorCode::CoordinatorEmptyProofData,
            50000 => ErrorCode::JWTCommonErr,
            50001 => ErrorCode::JWTTokenExpired,
            _ => {
                error!("get unexpected error code from coordinator: {v}");
                ErrorCode::Undefined(v)
            }
        }
    }
}

impl<'de> Deserialize<'de> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: i32 = i32::deserialize(deserializer)?;
        Ok(ErrorCode::from_i32(v))
    }
}
