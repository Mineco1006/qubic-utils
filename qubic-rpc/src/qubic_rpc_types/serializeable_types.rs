use qubic_rs::qubic_tcp_types::types::{ticks::CurrentTickInfo, Computors};
use qubic_rs::qubic_types::{QubicId, Signature};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputorInfos {
    pub epoch: u16,
    pub ids: Vec<QubicId>,
    pub signature: Signature,
}

impl From<Computors> for ComputorInfos {
    fn from(value: Computors) -> Self {
        ComputorInfos {
            epoch: value.epoch,
            ids: value.public_key.to_vec(),
            signature: value.signature,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestTick {
    pub latest_tick: u32,
}

impl From<CurrentTickInfo> for LatestTick {
    fn from(tick_info: CurrentTickInfo) -> Self {
        Self {
            latest_tick: tick_info.tick,
        }
    }
}