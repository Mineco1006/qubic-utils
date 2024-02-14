#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;
mod impls;
pub mod errors;
pub extern crate alloc;

#[cfg(feature = "serde")]
mod serde_impl;
pub mod traits;

pub use ethereum_types::{H256, H512, U256};


/// 32 byte nonce type
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Nonce(pub [u8; 32]);


/// 64 byte SchnorrQ signature type
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Signature(pub [u8; 64]);

impl Default for Signature {
    fn default() -> Self {
        Self([0; 64])
    }
}

/// Represents a Qubic ID containing only the decoded public key
/// 
/// # Initialization
/// ```
/// use qubic_types::QubicId;
/// let id_str = QubicId::from_str("BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK").unwrap(); // fails if ID is not valid
/// let id_public_key = QubicId([31, 89, 13, 3, 230, 19, 189, 222, 211, 139, 76, 8, 32, 172, 68, 97, 95, 145, 175, 18, 67, 89, 128, 179, 237, 227, 192, 140, 49, 90, 37, 68]); // inits ID from public key
/// 
/// assert_eq!(id_str.get_identity(), id_public_key.get_identity());
/// ```
/// 
/// # Verifying Signatures
/// ```
/// use qubic_types::{QubicId, Signature};
/// 
/// const SIGNATURE: Signature = Signature([200, 228, 166, 138, 90, 163, 195, 88, 137, 89, 233, 148, 251, 149, 140, 37, 105, 127, 254, 22, 49, 180, 202, 175, 236, 126, 224, 144, 41, 32, 119, 181, 96, 198, 20, 216, 126, 166, 96, 192, 252, 172, 247, 82, 47, 83, 49, 37, 227, 94, 186, 154, 189, 60, 111, 207, 59, 153, 206, 102, 219, 156, 24, 0]);
/// 
/// let id = QubicId::from_str("BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK").unwrap();
/// 
/// assert!(id.verify(10u64, SIGNATURE));
/// ```
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct QubicId(pub [u8; 32]);

/// Represents a Qubic wallet containing private key, subseed and public key of the corresponding wallet
/// 
/// # Initialization
/// ```
/// use qubic_types::QubicWallet;
/// 
/// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
/// 
/// assert_eq!(wallet.get_identity(), "BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK");
/// ```
/// 
/// ## Signing
/// 
/// ```
/// use qubic_types::QubicWallet;
/// 
/// let wallet = QubicWallet::from_seed("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
/// 
/// let data: u64 = 1006; // can be any data that derives std::marker::Copy or implements qubic_types::traits::AsByteEncoded
/// 
/// let signature = wallet.sign(&data);
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct QubicWallet {
    private_key: [u8; 32],
    subseed: [u8; 32],
    pub public_key: QubicId
}


#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct QubicTxHash(pub [u8; 32]);