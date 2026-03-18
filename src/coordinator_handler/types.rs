use super::error::ErrorCode;
use crate::prover::{ProofType, ProverProviderType};
use alloy_rlp::{
    BufMut, Decodable, EMPTY_STRING_CODE, Encodable, Header, RlpEncodable, length_of_length,
};
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};
use std::fmt;
use strum::{EnumIs, FromRepr};

#[derive(Debug, Clone)]
pub enum Response<T> {
    Ok(T),
    Err(RpcError),
}

impl<T> Response<T> {
    pub const fn is_ok(&self) -> bool {
        matches!(self, Response::Ok(_))
    }

    pub const fn is_error(&self) -> bool {
        matches!(self, Response::Err(_))
    }

    pub const fn is_jwt_token_expired(&self) -> bool {
        match self {
            Response::Err(err) => err.code.is_jwt_token_expired(),
            _ => false,
        }
    }

    pub const fn is_empty_task_error(&self) -> bool {
        match self {
            Response::Err(err) => err.code.is_coordinator_empty_proof_data(),
            _ => false,
        }
    }

    pub fn into_result(self) -> Result<T, RpcError> {
        match self {
            Response::Ok(data) => Ok(data),
            Response::Err(err) => Err(err),
        }
    }
}

#[derive(Deserialize)]
struct ResponseHelper<T> {
    errcode: ErrorCode,
    errmsg: String,
    data: Option<T>,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Response<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = ResponseHelper::<T>::deserialize(deserializer)?;
        if helper.errcode == ErrorCode::Success {
            if let Some(data) = helper.data {
                Ok(Response::Ok(data))
            } else if size_of::<T>() == 0 {
                // Special handling for zero-sized types, e.g. Reposnse<()>
                let zst: T = unsafe {
                    // SAFETY: It's always safe to synthesizing ZST
                    std::mem::zeroed()
                };
                Ok(Response::Ok(zst))
            } else {
                Err(serde::de::Error::custom(
                    "Expected data field for successful response",
                ))
            }
        } else {
            Ok(Response::Err(RpcError {
                code: helper.errcode,
                msg: helper.errmsg,
            }))
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RpcError {
    code: ErrorCode,
    msg: String,
}

impl RpcError {
    pub fn code(&self) -> &ErrorCode {
        &self.code
    }

    pub fn msg(&self) -> &str {
        &self.msg
    }
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.msg)
    }
}

impl std::error::Error for RpcError {}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    SerializeRepr,
    DeserializeRepr,
    EnumIs,
    strum::Display,
    FromRepr,
)]
#[repr(u8)]
pub enum ProverType {
    Undefined = 0,
    Chunk = 1,
    Batch = 2,
    OpenVM = 3,
}

impl Encodable for ProverType {
    fn encode(&self, s: &mut dyn BufMut) {
        (*self as u8).encode(s);
    }
}

impl Decodable for ProverType {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let v = u8::decode(buf)?;
        ProverType::from_repr(v).ok_or(alloy_rlp::Error::Custom("Invalid ProverType value"))
    }
}

/// The ProverType in go side is a type alias of uint8
/// A uint8 slice is treated as a string when doing the rlp encoding
#[derive(Debug, Clone, Serialize)]
pub struct ProverTypes<'a>(pub &'a [ProverType]);

/// See implementation of [`Encodable`] for [u8]
impl Encodable for ProverTypes<'_> {
    #[inline]
    fn encode(&self, out: &mut dyn BufMut) {
        if self.0.len() != 1 || self.0[0] as u8 >= EMPTY_STRING_CODE {
            Header {
                list: false,
                payload_length: self.0.len(),
            }
            .encode(out);
        }
        for prover_type in self.0.iter() {
            out.put_u8(*prover_type as u8);
        }
    }
    #[inline]
    fn length(&self) -> usize {
        let mut len = self.0.len();
        if len != 1 || self.0[0] as u8 >= EMPTY_STRING_CODE {
            len += length_of_length(len);
        }
        len
    }
}

#[derive(Debug, Clone, Serialize, RlpEncodable)]
pub struct LoginMessage<'a> {
    pub challenge: &'a str,
    pub prover_version: &'a str,
    pub prover_name: &'a str,
    pub prover_provider_type: ProverProviderType,
    pub prover_types: &'a Vec<ProverType>,
    pub vks: &'a Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest<'a> {
    pub message: LoginMessage<'a>,
    pub public_key: &'a str,
    pub signature: &'a str,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponse {
    pub time: String,
    pub token: String,
}

pub type ChallengeResponse = LoginResponse;

#[derive(Default, Debug, Clone, Serialize)]
pub struct GetTaskRequest<'a> {
    pub task_types: Vec<ProofType>,
    pub prover_height: Option<u64>,
    pub universal: bool,
    pub task_id: Option<&'a str>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct GetTaskResponse {
    pub uuid: String,
    pub task_id: String,
    pub task_type: ProofType,
    pub task_data: String,
    pub hard_fork_name: String,
}

#[derive(Debug, Clone, Serialize)] // TODO: Default?
pub struct SubmitProofRequest<'a> {
    pub uuid: &'a str,
    pub task_id: &'a str,
    pub task_type: ProofType,
    pub status: ProofStatus,
    pub proof: &'a str,
    pub failure_type: Option<ProofFailureType>,
    pub failure_msg: Option<&'a str>,
    pub universal: bool,
}

#[derive(
    Default,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    SerializeRepr,
    DeserializeRepr,
    EnumIs,
    strum::Display,
    FromRepr,
)]
#[repr(u8)]
pub enum ProofFailureType {
    #[default]
    Undefined = 0,
    Panic = 1,
    NoPanic = 2,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    SerializeRepr,
    DeserializeRepr,
    EnumIs,
    strum::Display,
    FromRepr,
)]
#[repr(u8)]
pub enum ProofStatus {
    Ok = 0,
    Error = 1,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coordinator_handler::KeySigner, prover::types::ProverProviderType};

    #[test]
    fn test_prover_provider_type_encoding() {
        // Test that ProverProviderType values match the coordinator's values
        assert_eq!(ProverProviderType::Undefined as u8, 0);
        assert_eq!(ProverProviderType::Internal as u8, 1);
        assert_eq!(ProverProviderType::External as u8, 2);
    }

    // This test uses the same private key as the coordinator's TestGenerateSignature
    // to verify signature generation compatibility
    #[test]
    fn test_signature_compatibility() {
        let private_key_hex = "8b8df68fddf7ee2724b79ccbd07799909d59b4dd4f4df3f6ecdc4fb8d56bdf4c";
        let key_signer = KeySigner::new_from_secret_key(private_key_hex).unwrap();

        let prover_types = vec![ProverType::Chunk];
        let vks = vec!["mock_vk".to_string()];
        let login_message = LoginMessage {
            challenge: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MjQ4Mzg0ODUsIm9yaWdfaWF0IjoxNzI0ODM0ODg1LCJyYW5kb20iOiJ6QmdNZGstNGc4UzNUNTFrVEFsYk1RTXg2TGJ4SUs4czY3ejM2SlNuSFlJPSJ9.x9PvihhNx2w4_OX5uCrv8QJCNYVQkIi-K2k8XFXYmik".into(),
            prover_version: "v4.4.45-37af5ef5-38a68e2-1c5093c".into(),
            prover_name: "test".into(),
            prover_provider_type: ProverProviderType::Internal,
            prover_types: &prover_types,
            vks: &vks,
        };

        let buffer = alloy_rlp::encode(&login_message);
        let signature = key_signer
            .sign_buffer(&buffer)
            .map_err(|e| eyre::eyre!("Failed to sign the login message: {e}"))
            .unwrap();

        // expected signature from coordinator's TestGenerateSignature
        let expected_signature = "0xb8659f094fde9ed697bd86b8d8a0a1cff902710d7750463858c8a9ff9e851b152240054f256ce9ea8a3eaf5f0d56ceed894b358d3505926dc6cfc36548f7001a01".to_string();
        assert_eq!(signature, expected_signature);
    }
}
