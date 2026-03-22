use alloy_rlp::{BufMut, Decodable, Encodable};
use serde_repr::{Deserialize_repr as DeserializeRepr, Serialize_repr as SerializeRepr};
use strum::{Display, EnumIs, FromRepr};

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
    Display,
    FromRepr,
)]
#[repr(u8)]
pub enum ProverProviderType {
    #[default]
    Undefined = 0,
    Internal = 1,
    External = 2,
}

impl Encodable for ProverProviderType {
    fn encode(&self, out: &mut dyn BufMut) {
        (*self as u8).encode(out);
    }
}

impl Decodable for ProverProviderType {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let v = u8::decode(buf)?;
        Ok(ProverProviderType::from_repr(v).unwrap_or(ProverProviderType::Undefined))
    }
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
    Display,
    FromRepr,
)]
#[repr(u8)]
pub enum ProofType {
    #[default]
    Undefined = 0,
    Chunk = 1,
    Batch = 2,
    Bundle = 3,
}

impl Encodable for ProofType {
    fn encode(&self, out: &mut dyn BufMut) {
        (*self as u8).encode(out);
    }
}

impl Decodable for ProofType {
    fn decode(buf: &mut &[u8]) -> alloy_rlp::Result<Self> {
        let v = u8::decode(buf)?;
        Ok(ProofType::from_repr(v).unwrap_or(ProofType::Undefined))
    }
}
