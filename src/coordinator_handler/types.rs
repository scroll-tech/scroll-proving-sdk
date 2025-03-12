use super::error::ErrorCode;
use crate::{
    prover::{ProofType, ProverProviderType},
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProverType {
    Undefined,
    Chunk,
    Batch,
    OpenVM,
}

impl ProverType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => ProverType::Chunk,
            2 => ProverType::Batch,
            3 => ProverType::OpenVM,
            _ => ProverType::Undefined,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            ProverType::Undefined => 0,
            ProverType::Chunk => 1,
            ProverType::Batch => 2,
            ProverType::OpenVM => 3,
        }
    }
}

impl Serialize for ProverType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ProverType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(ProverType::from_u8(v))
    }
}

#[derive(Serialize, Deserialize)]
pub struct LoginMessage {
    pub challenge: String,
    pub prover_version: String,
    pub prover_name: String,
    pub prover_provider_type: ProverProviderType,
    pub prover_types: Vec<ProverType>,
    pub vks: Vec<String>,
}

impl Encodable for LoginMessage {
    fn rlp_append(&self, s: &mut RlpStream) {
        let num_fields = 6;
        s.begin_list(num_fields);
        s.append(&self.challenge);
        s.append(&self.prover_version);
        s.append(&self.prover_name);
        s.append(&(self.prover_provider_type as u8));
        // The ProverType in go side is an type alias of uint8
        // A uint8 slice is treated as a string when doing the rlp encoding
        let prover_types = self
            .prover_types
            .iter()
            .map(|prover_type| prover_type.to_u8())
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
    pub task_types: Vec<ProofType>,
    pub prover_height: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTaskResponseData {
    pub uuid: String,
    pub task_id: String,
    pub task_type: ProofType,
    pub task_data: String,
    pub hard_fork_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChunkTaskDetail {
    pub block_hashes: Vec<CommonHash>,
    pub prev_msg_queue_hash: CommonHash,
}

#[derive(Serialize, Deserialize)] // TODO: Default?
pub struct SubmitProofRequest {
    pub uuid: String,
    pub task_id: String,
    pub task_type: ProofType,
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

        let login_message = LoginMessage {
            challenge: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJleHAiOjE3MjQ4Mzg0ODUsIm9yaWdfaWF0IjoxNzI0ODM0ODg1LCJyYW5kb20iOiJ6QmdNZGstNGc4UzNUNTFrVEFsYk1RTXg2TGJ4SUs4czY3ejM2SlNuSFlJPSJ9.x9PvihhNx2w4_OX5uCrv8QJCNYVQkIi-K2k8XFXYmik".to_string(),
            prover_version: "v4.4.45-37af5ef5-38a68e2-1c5093c".to_string(),
            prover_name: "test".to_string(),
            prover_provider_type: ProverProviderType::Internal,
            prover_types: vec![ProverType::Chunk],
            vks: vec!["mock_vk".to_string()],
        };

        let buffer = rlp::encode(&login_message);
        let signature = key_signer
            .sign_buffer(&buffer)
            .map_err(|e| anyhow::anyhow!("Failed to sign the login message: {e}"))
            .unwrap();

        // expected signature from coordinator's TestGenerateSignature
        let expected_signature = "0xb8659f094fde9ed697bd86b8d8a0a1cff902710d7750463858c8a9ff9e851b152240054f256ce9ea8a3eaf5f0d56ceed894b358d3505926dc6cfc36548f7001a01".to_string();
        assert_eq!(signature, expected_signature);
    }
}
