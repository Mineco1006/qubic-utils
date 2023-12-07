use std::fmt::{Debug, Display};

use qubic_types::{QubicId, Signature};

use crate::MessageType;

use super::transactions::Transaction;

pub const QXID: QubicId = QubicId([1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Name([u8; 7]);

impl ToString for Name {
    fn to_string(&self) -> String {
        let mut name = String::with_capacity(7);

        for byte in self.0 {
            if byte != 0 {
                name.push(char::from(byte));
            }
        }

        name
    }
}

impl Debug for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Issuance {
    pub name: Name,
    pub number_of_decimal_places: u8,
    pub unit_of_measurement: [u8; 7]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Ownership {
    pub padding: u8,
    pub managing_contract_index: [u8; 2],
    pub issuance_index: [u8; 4],
    pub number_of_units: [u8; 8]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Possession {
    pub padding: u8,
    pub managing_contract_index: [u8; 2],
    pub issuance_index: [u8; 4],
    pub number_of_units: [u8; 8]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8, C)]
pub enum AssetType {
    Empty = 0,
    Issuance(Issuance) = 1,
    Ownership(Ownership) = 2,
    Possession(Possession) = 3
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct Asset {
    pub public_key: QubicId,
    pub asset_type: AssetType
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FeesInput;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct FeesOutput {
    pub asset_issuance_fee: u32,
    pub transfer_fee: u32,
    pub trade_fee: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IssueAssetInput {
    pub name: u64,
    pub number_of_units: i64,
    pub unit_of_measurement: u64,
    pub number_of_decimal_places: i8
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct IssueAssetOutput {
    pub issued_number_of_units: i64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TranferAssetOwnershipAndPossessionInput {
    pub transaction: Transaction,

    pub issuer: QubicId,
    pub possessor: QubicId,
    pub new_owner: QubicId,
    pub asset_name: u64,
    pub number_of_units: i64,

    pub signature: Signature
}

set_message_type!(TranferAssetOwnershipAndPossessionInput, MessageType::BroadcastTransaction);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct TranferAssetOwnershipAndPossessionOutput {
    pub transferred_number_of_units: i64
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestIssuedAsset {
    pub public_key: QubicId
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RespondIssuedAsset {
    pub asset: Asset,
    pub tick: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestOwnedAsset {
    pub public_key: QubicId
}

set_message_type!(RequestOwnedAsset, MessageType::RequestOwnedAsset);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, align(8))]
pub struct RespondOwnedAsset {
    pub asset: Asset,
    pub issuance_asset: Asset,
    pub tick: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RequestPossessedAsset {
    pub public_key: QubicId
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct RespondPossessedAsset {
    pub asset: Asset,
    pub ownership_asset: Asset,
    pub issuance_asset: Asset,
    pub tick: u32
}

#[test]
fn test() {
    dbg!(std::mem::size_of::<Asset>());
}