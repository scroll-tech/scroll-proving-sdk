use super::error::ErrorCode;
use crate::{
    prover::{CircuitType, ProverProviderType},
    tracing_handler::CommonHash,
};
use rlp::{Encodable, RlpStream};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Deserialize)]
pub struct Response<T> {
    pub errcode: ErrorCode,
    pub errmsg: String,
    pub data: Option<T>,
}

#[derive(Serialize, Deserialize)]
pub struct LoginMessage {
    pub challenge: String,
    pub prover_version: String,
    pub prover_name: String,
    pub prover_provider_type: ProverProviderType,
    pub prover_types: Vec<CircuitType>,
    pub vks: Vec<String>,
}

impl Encodable for LoginMessage {
    fn rlp_append(&self, s: &mut RlpStream) {
        let num_fields = 5;
        s.begin_list(num_fields);
        s.append(&self.challenge);
        s.append(&self.prover_version);
        s.append(&self.prover_name);
        // The ProverType in go side is an type alias of uint8
        // A uint8 slice is treated as a string when doing the rlp encoding
        let prover_types = self
            .prover_types
            .iter()
            .map(|prover_type: &CircuitType| prover_type.to_u8())
            .collect::<Vec<u8>>();
        s.append(&prover_types);
        s.begin_list(self.vks.len());
        for vk in &self.vks {
            s.append(vk);
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginRequest {
    pub message: LoginMessage,
    pub public_key: String,
    pub signature: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginResponseData {
    pub time: String,
    pub token: String,
}

pub type ChallengeResponseData = LoginResponseData;

#[derive(Default, Serialize, Deserialize)]
pub struct GetTaskRequest {
    pub task_types: Vec<CircuitType>,
    pub prover_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTaskResponseData {
    pub uuid: String,
    pub task_id: String,
    pub task_type: CircuitType,
    pub task_data: String,
    pub hard_fork_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChunkTaskDetail {
    pub block_hashes: Vec<CommonHash>,
}

#[derive(Serialize, Deserialize)] // TODO: Default?
pub struct SubmitProofRequest {
    pub uuid: String,
    pub task_id: String,
    pub task_type: CircuitType,
    pub status: ProofStatus,
    pub proof: String,
    pub failure_type: Option<ProofFailureType>,
    pub failure_msg: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SubmitProofResponseData {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProofFailureType {
    Undefined,
    Panic,
    NoPanic,
}

impl ProofFailureType {
    fn from_u8(v: u8) -> Self {
        match v {
            1 => ProofFailureType::Panic,
            2 => ProofFailureType::NoPanic,
            _ => ProofFailureType::Undefined,
        }
    }
}

impl Serialize for ProofFailureType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ProofFailureType::Undefined => serializer.serialize_u8(0),
            ProofFailureType::Panic => serializer.serialize_u8(1),
            ProofFailureType::NoPanic => serializer.serialize_u8(2),
        }
    }
}

impl<'de> Deserialize<'de> for ProofFailureType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(ProofFailureType::from_u8(v))
    }
}

impl Default for ProofFailureType {
    fn default() -> Self {
        Self::Undefined
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProofStatus {
    Ok,
    Error,
}

impl ProofStatus {
    fn from_u8(v: u8) -> Self {
        match v {
            0 => ProofStatus::Ok,
            _ => ProofStatus::Error,
        }
    }
}

impl Serialize for ProofStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ProofStatus::Ok => serializer.serialize_u8(0),
            ProofStatus::Error => serializer.serialize_u8(1),
        }
    }
}

impl<'de> Deserialize<'de> for ProofStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(ProofStatus::from_u8(v))
    }
}
