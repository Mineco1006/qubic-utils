use crate::MessageType;


pub trait AsByteEncoded where Self: Sized {
    fn encode_as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((self as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }
}

pub trait GetMessageType {
    fn get_message_type(&self) -> MessageType; 
}