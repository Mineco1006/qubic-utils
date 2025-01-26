use qubic_types::QubicId;

pub const SEND_TO_MANY_CONTRACT_INDEX: u32 = 4;


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(C)]
pub struct SendToManyInput {
    pub ids: [QubicId; 25],
    pub amounts: [u64; 25]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SendToManyTransaction {
    pub id: QubicId,
    pub amount: u64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct SendToManyFeeOutput {
    pub fee: u32
}