use crate::MessageType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestContractFunction {
    pub contract_index: u32,
    pub input_type: u16,
    pub input_size: u16
}

set_message_type!(RequestContractFunction, MessageType::RequestContractFunction);