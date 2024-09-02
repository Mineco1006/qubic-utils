use core::str::FromStr;
use alloc::format;

use serde::{Serialize, Deserialize, de::Visitor};

use crate::{QubicId, Signature, MiningSeed, Nonce, QubicTxHash};


struct QubicIdVisitor;

impl<'de> Visitor<'de> for QubicIdVisitor {
    type Value = QubicId;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("60 uppercase character alphabetic ASCII string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
        match QubicId::from_str(value) {
            Ok(r) => Ok(r),
            Err(e) => {
                Err(E::custom(e.to_string()))
            } 
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: serde::de::Error, {
        match QubicId::from_str(&value) {
            Ok(r) => Ok(r),
            Err(e) => {
                Err(E::custom(e.to_string()))
            } 
        }
    }
}

impl Serialize for QubicId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_str(&self.get_identity())
    }
}

impl<'de> Deserialize<'de> for QubicId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_str(QubicIdVisitor)
    }
}

struct QubicTxHashVisitor;

impl<'de> Visitor<'de> for QubicTxHashVisitor {
    type Value = QubicTxHash;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("60 lowercase character alphabetic ASCII string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
        match QubicTxHash::from_str(value) {
            Ok(r) => Ok(r),
            Err(e) => {
                Err(E::custom(e.to_string()))
            } 
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> where E: serde::de::Error, {
        match QubicTxHash::from_str(&value) {
            Ok(r) => Ok(r),
            Err(e) => {
                Err(E::custom(e.to_string()))
            } 
        }
    }
}

impl Serialize for QubicTxHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_str(&self.get_identity())
    }
}

impl<'de> Deserialize<'de> for QubicTxHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserializer.deserialize_str(QubicTxHashVisitor)
    }
}


struct HexVisitor<const LENGTH: usize>;

impl<'de, const LENGTH: usize> Visitor<'de> for HexVisitor<LENGTH> {
    type Value = [u8; LENGTH];

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("0x prefixed hexadecimal string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: serde::de::Error, {
        if !v.starts_with("0x") {
            return Err(E::custom("string is not 0x prefixed"))
        }

        match hex::decode(&v[2..]) {
            Ok(r) => {
                if r.len() != LENGTH {
                    Err(E::custom("invalid length"))
                } else {
                    Ok(r.try_into().unwrap())
                }
            },
            Err(e) => {
                Err(E::custom(e.to_string()))
            }
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: serde::de::Error, {
        if !v.starts_with("0x") {
            return Err(E::custom("string is not 0x prefixed"))
        }

        match hex::decode(&v[2..]) {
            Ok(r) => {
                if r.len() != LENGTH {
                    Err(E::custom("invalid length"))
                } else {
                    Ok(r.try_into().unwrap())
                }
            },
            Err(e) => {
                Err(E::custom(e.to_string()))
            }
        }
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Ok(Signature(deserializer.deserialize_str(HexVisitor)?))
    }
}

impl Serialize for MiningSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for MiningSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Ok(MiningSeed(deserializer.deserialize_str(HexVisitor)?))
    }
}

impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.collect_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        Ok(Nonce(deserializer.deserialize_str(HexVisitor)?))
    }
}