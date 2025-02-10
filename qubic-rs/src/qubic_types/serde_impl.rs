use alloc::format;
use core::str::FromStr;

use serde::{de::Visitor, Deserialize, Serialize};

use crate::{
    qubic_tcp_types::types::ContractIpo,
    qubic_types::{MiningSeed, Nonce, QubicId, QubicTxHash, Signature},
};

struct QubicIdVisitor;

impl<'de> Visitor<'de> for QubicIdVisitor {
    type Value = QubicId;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("60 uppercase character alphabetic ASCII string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match QubicId::from_str(value) {
            Ok(r) => Ok(r),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match QubicId::from_str(&value) {
            Ok(r) => Ok(r),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl Serialize for QubicId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.get_identity())
    }
}

impl<'de> Deserialize<'de> for QubicId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(QubicIdVisitor)
    }
}

impl Serialize for ContractIpo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeTuple;

        let mut seq = serializer.serialize_tuple(676)?;
        for item in &self.public_keys {
            seq.serialize_element(item)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ContractIpo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{SeqAccess, Visitor};
        use std::fmt;

        struct ContractIpoVisitor;

        impl<'de> Visitor<'de> for ContractIpoVisitor {
            type Value = ContractIpo;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a struct representing ContractIpo")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let contract_index = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let tick = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                let mut public_keys = [QubicId::default(); 676];
                for i in 0..676 {
                    public_keys[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2 + i, &self))?;
                }

                let mut prices = [0u64; 676];
                for i in 0..676 {
                    prices[i] = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(2 + 676 + i, &self))?;
                }

                Ok(ContractIpo {
                    contract_index,
                    tick,
                    public_keys,
                    prices,
                })
            }
        }

        deserializer.deserialize_seq(ContractIpoVisitor)
    }
}

struct LowerCaseIdentityVisitor;

impl<'de> Visitor<'de> for LowerCaseIdentityVisitor {
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("60 lowercase character alphabetic ASCII string")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match QubicTxHash::from_str(value) {
            Ok(r) => Ok(r.0),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match QubicTxHash::from_str(&value) {
            Ok(r) => Ok(r.0),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl Serialize for QubicTxHash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&self.get_identity())
    }
}

impl<'de> Deserialize<'de> for QubicTxHash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(QubicTxHash(
            deserializer.deserialize_str(LowerCaseIdentityVisitor)?,
        ))
    }
}

struct HexVisitor<const LENGTH: usize>;

impl<'de, const LENGTH: usize> Visitor<'de> for HexVisitor<LENGTH> {
    type Value = [u8; LENGTH];

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("0x prefixed hexadecimal string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if !v.starts_with("0x") {
            return Err(E::custom("string is not 0x prefixed"));
        }

        match hex::decode(&v[2..]) {
            Ok(r) => {
                if r.len() != LENGTH {
                    Err(E::custom("invalid length"))
                } else {
                    Ok(r.try_into().unwrap())
                }
            }
            Err(e) => Err(E::custom(e.to_string())),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if !v.starts_with("0x") {
            return Err(E::custom("string is not 0x prefixed"));
        }

        match hex::decode(&v[2..]) {
            Ok(r) => {
                if r.len() != LENGTH {
                    Err(E::custom("invalid length"))
                } else {
                    Ok(r.try_into().unwrap())
                }
            }
            Err(e) => Err(E::custom(e.to_string())),
        }
    }
}

impl Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Signature(deserializer.deserialize_str(HexVisitor)?))
    }
}

impl Serialize for MiningSeed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format!("{}", self.get_identity()))
    }
}

impl<'de> Deserialize<'de> for MiningSeed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(MiningSeed(
            deserializer.deserialize_str(LowerCaseIdentityVisitor)?,
        ))
    }
}

impl Serialize for Nonce {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&format!("0x{}", hex::encode(self.0)))
    }
}

impl<'de> Deserialize<'de> for Nonce {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(Nonce(deserializer.deserialize_str(HexVisitor)?))
    }
}