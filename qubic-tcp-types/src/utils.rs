use crate::MessageType;

pub trait QubicRequest {
    fn get_message_type(&self) -> MessageType; 
}

pub trait QubicReturnType {
    type ReturnType;
}