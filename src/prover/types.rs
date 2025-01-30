use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CircuitType {
    #[default]
    Undefined,
    Halo2,
    OpenVM,
}

impl CircuitType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => CircuitType::Halo2,
            2 => CircuitType::OpenVM,
            _ => CircuitType::Undefined,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            CircuitType::Undefined => 0,
            CircuitType::Halo2 => 1,
            CircuitType::OpenVM => 2,
        }
    }
}

impl Serialize for CircuitType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for CircuitType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(CircuitType::from_u8(v))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum ProverProviderType {
    #[default]
    Undefined,
    Internal,
    External,
}

impl ProverProviderType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => ProverProviderType::Internal,
            2 => ProverProviderType::External,
            _ => ProverProviderType::Undefined,
        }
    }
}

impl Serialize for ProverProviderType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            ProverProviderType::Undefined => serializer.serialize_u8(0),
            ProverProviderType::Internal => serializer.serialize_u8(1),
            ProverProviderType::External => serializer.serialize_u8(2),
        }
    }
}

impl<'de> Deserialize<'de> for ProverProviderType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(ProverProviderType::from_u8(v))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProofType {
    #[default]
    Undefined,
    Chunk,
    Batch,
    Bundle,
}

impl ProofType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => ProofType::Chunk,
            2 => ProofType::Batch,
            3 => ProofType::Bundle,
            _ => ProofType::Undefined,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            ProofType::Undefined => 0,
            ProofType::Chunk => 1,
            ProofType::Batch => 2,
            ProofType::Bundle => 3,
        }
    }
}

impl Serialize for ProofType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u8(self.to_u8())
    }
}

impl<'de> Deserialize<'de> for ProofType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: u8 = u8::deserialize(deserializer)?;
        Ok(ProofType::from_u8(v))
    }
}
