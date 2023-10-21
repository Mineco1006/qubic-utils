use serde::{Serialize, Deserialize, de::Visitor};

use crate::{QubicId, Signature};


struct QubicIdVisitor;

impl<'de> Visitor<'de> for QubicIdVisitor {
    type Value = QubicId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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

struct HexVisitor<const LENGTH: usize>;

impl<'de, const LENGTH: usize> Visitor<'de> for HexVisitor<LENGTH> {
    type Value = [u8; LENGTH];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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