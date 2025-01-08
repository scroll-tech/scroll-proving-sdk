use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CircuitType {
    #[default]
    Undefined,
    Chunk,
    Batch,
    Bundle,
}

impl CircuitType {
    pub fn from_u8(v: u8) -> Self {
        match v {
            1 => CircuitType::Chunk,
            2 => CircuitType::Batch,
            3 => CircuitType::Bundle,
            _ => CircuitType::Undefined,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            CircuitType::Undefined => 0,
            CircuitType::Chunk => 1,
            CircuitType::Batch => 2,
            CircuitType::Bundle => 3,
        }
    }
}

impl Serialize for CircuitType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            CircuitType::Undefined => serializer.serialize_u8(0),
            CircuitType::Chunk => serializer.serialize_u8(1),
            CircuitType::Batch => serializer.serialize_u8(2),
            CircuitType::Bundle => serializer.serialize_u8(3),
        }
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
