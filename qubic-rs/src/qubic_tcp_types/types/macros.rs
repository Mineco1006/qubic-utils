macro_rules! set_message_type {
    ($impl: ident, $message_type: expr) => {
        impl crate::qubic_tcp_types::utils::QubicRequest for $impl {
            fn get_message_type() -> MessageType {
                $message_type
            }
        }
    };
}