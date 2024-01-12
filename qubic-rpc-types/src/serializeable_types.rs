use qubic_types::{QubicId, Signature};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputorInfos {
    pub epoch: u16,
    pub ids: Vec<QubicId>,
    pub signature: Signature
}

impl Into<ComputorInfos> for qubic_tcp_types::types::Computors {
    fn into(self) -> ComputorInfos {
        ComputorInfos {
            epoch: self.epoch,
            ids: self.public_key.to_vec(),
            signature: self.signature
        }
    }
}