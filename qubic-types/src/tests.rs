use crate::{QubicId, QubicWallet};

const SEED: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const ID: &str = "BZBQFLLBNCXEMGLOBHUVFTLUPLVCPQUASSILFABOFFBCADQSSUPNWLZBQEXK";

/// Test public key generation from 60 character ID
#[test]
pub fn test_id() {
    let pk = QubicId::from_str(ID).unwrap();

    assert_eq!(pk.0, [31, 89, 13, 3, 230, 19, 189, 222, 211, 139, 76, 8, 32, 172, 68, 97, 95, 145, 175, 18, 67, 89, 128, 179, 237, 227, 192, 140, 49, 90, 37, 68]);
}


// Test wallet signature & public key generation from 55 character seed
#[test]
pub fn test_wallet() {
    let wallet = QubicWallet::from_seed(SEED).unwrap();

    assert_eq!(wallet.get_identity(), ID);

    let signature = wallet.sign(10u64);

    assert_eq!(signature.0, [200, 228, 166, 138, 90, 163, 195, 88, 137, 89, 233, 148, 251, 149, 140, 37, 105, 127, 254, 22, 49, 180, 202, 175, 236, 126, 224, 144, 41, 32, 119, 181, 96, 198, 20, 216, 126, 166, 96, 192, 252, 172, 247, 82, 47, 83, 49, 37, 227, 94, 186, 154, 189, 60, 111, 207, 59, 153, 206, 102, 219, 156, 24, 0]);
    
    let id = QubicId::from_str(ID).unwrap();

    assert!(id.verify(10u64, signature));
}