use qubic_tcp_types::types::Computors;
use qubic_types::{QubicId, Signature};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputorInfos {
    pub epoch: u16,
    pub ids: Vec<QubicId>,
    pub signature: Signature
}

impl From<Computors> for ComputorInfos {
    fn from(value: Computors) -> Self {
        ComputorInfos {
            epoch: value.epoch,
            ids: value.public_key.to_vec(),
            signature: value.signature
        }
    }
}