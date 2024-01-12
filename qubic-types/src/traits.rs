use std::ptr::read_unaligned;
use kangarootwelve::KangarooTwelve;
use crate::Signature;

use crate::{errors::ByteEncodingError, QubicId};

pub trait ToBytes {
    fn to_bytes(&self) -> Vec<u8>;
}

pub trait FromBytes where Self: Sized {
    fn from_bytes(data: &[u8]) -> Result<Self, ByteEncodingError>;
}

impl<T: Copy> ToBytes for T {
    fn to_bytes(&self) -> Vec<u8> {
        unsafe {
            core::slice::from_raw_parts(self as *const T as *const u8, std::mem::size_of::<T>()).to_vec()
        }
    }
}

impl<T: Copy> FromBytes for T {
    fn from_bytes(data: &[u8]) -> Result<Self, ByteEncodingError> {
        if data.len() != std::mem::size_of::<Self>() {
            return Err(ByteEncodingError::InvalidDataLength { expected: std::mem::size_of::<Self>(), found: data.len() })
        }

        Ok(
            unsafe {
                read_unaligned(data.as_ptr() as *const T)
            }
        )
    }
}

pub trait GetSigner {
    fn get_signer(&self) -> &QubicId;
}

pub trait VerifySignature {
    fn verify(&self) -> bool;
}

impl<T: ToBytes + GetSigner> VerifySignature for T {
    fn verify(&self) -> bool {
        let mut digest = [0; 32];

        let bytes = self.to_bytes();
        let signature = Signature(bytes[bytes.len() - std::mem::size_of::<Signature>()..bytes.len()].try_into().unwrap());
        let mut kg = KangarooTwelve::hash(&bytes[..bytes.len() - std::mem::size_of::<Signature>()], &[]);
        kg.squeeze(&mut digest);

        self.get_signer().verify_raw(digest, signature)
    }
}