pub trait AsByteEncoded where Self: Sized {
    fn encode_as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts((self as *const Self) as *const u8, core::mem::size_of::<Self>())
        }
    }
}

impl<T: Sized> AsByteEncoded for T {}