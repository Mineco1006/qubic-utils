use crate::qubic_tcp_types::MessageType;
use crate::qubic_types::traits::{FromBytes, ToBytes};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestContractFunction {
    pub contract_index: u32,
    pub input_type: u16,
    pub input_size: u16,
    pub input: Vec<u8>,
}

set_message_type!(
    RequestContractFunction,
    MessageType::RequestContractFunction
);

impl ToBytes for RequestContractFunction {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.contract_index.to_bytes();
        bytes.extend(self.input_type.to_bytes());
        bytes.extend(self.input_size.to_bytes());
        bytes.extend(self.input.clone());
        bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct ResponseContractFunction {
    pub output: Vec<u8>,
}

set_message_type!(
    ResponseContractFunction,
    MessageType::RespondContractFunction
);

impl FromBytes for ResponseContractFunction {
    fn from_bytes(data: &[u8]) -> Result<Self, crate::qubic_types::errors::ByteEncodingError> {
        Ok(Self {
            output: data.to_vec(),
        })
    }
}