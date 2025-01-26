use crate::MessageType;

pub trait QubicRequest {
    fn get_message_type() -> MessageType; 
}

pub trait QubicReturnType {
    type ReturnType;
}