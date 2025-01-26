

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum QubicLogType {
    QuTransfer = 0,
    AssetIssuance = 1,
    AssetOwnershipChange = 2,
    AssetPossessionChange = 3,
    ContractErrorMessage = 4,
    ContractWarningMessage = 5,
    ContractInformationMessage = 6,
    ContractDebugMessage = 7,
    CustomMessage = 255,
    #[default]
    None = 254
}

impl QubicLogType {
    pub fn get_expected_size(&self) -> Option<usize> {
        match self {
            Self::QuTransfer => Some(72),
            Self::AssetIssuance => Some(55),
            Self::AssetOwnershipChange => Some(119),
            Self::AssetPossessionChange => Some(119),
            Self::ContractErrorMessage => Some(4),
            _ => None
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct RequestLog {
    pub passcode: [u64; 4]
}

set_message_type!(RequestLog, MessageType::RequestLog);

pub struct RespondLog {
    pub log_bytes: Vec<u8>
}

use core::fmt::{Debug, Display};

use qubic_types::{errors::ByteEncodingError, traits::FromBytes, QubicId};

use crate::{types::assets::AssetName, MessageType};


#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub struct LogHeader {
    pub year: u8,
    pub month: u8,
    pub day: u8,

    pub hour: u8,
    pub minute: u8,
    pub second: u8,

    pub epoch: u8,
    pub tick: u32,
    pub size: [u8; 3],
    pub log_type: QubicLogType
}

impl LogHeader {
    pub fn get_size(&self) -> usize {
        self.size[0] as usize | (self.size[1] as usize) << 8 | (self.size[2] as usize) << 16
    }
}

impl Debug for LogHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("[{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2} EP{}@{} {:?}]", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second, self.epoch, self.tick, self.log_type))
    }
}

impl Display for LogHeader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("[{}/{:0>2}/{:0>2} {:0>2}:{:0>2}:{:0>2} EP{}@{} {:?}]", 2000 + self.year as u16, self.month, self.day, self.hour, self.minute, self.second, self.epoch, self.tick, self.log_type))
    }
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct QuTransferLog {
    pub from: QubicId,
    pub to: QubicId,
    pub amount: u64,
    pub transfer_id: Option<u64>
}

impl FromBytes for QuTransferLog {
    fn from_bytes(data: &[u8]) -> Result<Self, ByteEncodingError> {
        if data.len() < 72 {
            return Err(ByteEncodingError::InvalidMinimumDataLength { expected_min: 72, found: data.len() })
        }


        Ok(Self {
            from: QubicId::from_bytes(&data[..32])?,
            to: QubicId::from_bytes(&data[32..64])?,
            amount: u64::from_bytes(&data[64..72])?,
            transfer_id: if data.len() == 72 { None } else { Some(u64::from_bytes(&data[72..80])?) }
        })
    }
}

impl Display for QuTransferLog {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("Transfer {} -> {} | Amount: {} QUs | Transfer ID: {:?}", self.from, self.to, self.amount, self.transfer_id))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetIssuanceLog {
    pub from: QubicId,
    pub number_of_shares: u64,
    pub name: AssetName<7>,
    pub number_of_decimal_places: u8,
    pub unit: [u8; 7],
}

impl Display for AssetIssuanceLog {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("New Asset {} issued by {} | Shares: {}", self.from, self.name.to_string(), self.number_of_shares as f32 / self.number_of_decimal_places as f32))
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetOwnershipChangeLog {
    pub from: QubicId,
    pub to: QubicId,
    pub issuer: QubicId,
    pub number_of_shares: u64,
    pub name: AssetName<7>,
    pub number_of_decimal_places: u8,
    pub unit_of_measurement: [u8; 7],
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct AssetPossessionChangeLog {
    pub from: QubicId,
    pub to: QubicId,
    pub issuer: QubicId,
    pub number_of_shares: u64,
    pub name: AssetName<7>,
    pub number_of_decimal_places: u8,
    pub unit_of_measurement: [u8; 7],
}


#[derive(Debug, Clone, Default)]
pub enum LogMessages {
    QuTransferLog(QuTransferLog),
    AssetIssuanceLog(AssetIssuanceLog),
    AssetOwnershipChangeLog(AssetOwnershipChangeLog),
    AssetPossessionChangeLog(AssetPossessionChangeLog),
    String(String),
    #[default]
    None
}

impl Display for LogMessages {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::QuTransferLog(log) => f.write_fmt(format_args!("{log}")),
            Self::AssetIssuanceLog(log) => f.write_fmt(format_args!("{log}")),
            Self::AssetOwnershipChangeLog(log) => f.write_fmt(format_args!("{log:?}")),
            Self::AssetPossessionChangeLog(log) => f.write_fmt(format_args!("{log:?}")),
            Self::String(log) => f.write_fmt(format_args!("{log}")),
            Self::None => f.write_str("")
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct QubicLog {
    pub header: LogHeader,
    pub message: LogMessages
}

impl Display for QubicLog {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{} {}", self.header, self.message))
    }
}

impl FromBytes for QubicLog {
    fn from_bytes(data: &[u8]) -> Result<Self, qubic_types::errors::ByteEncodingError> {
        if data.len() == 0 {
            return Ok(Self::default())
        }

        let header = LogHeader::from_bytes(&data[..core::mem::size_of::<LogHeader>()])?;

        dbg!(header.get_size());
        dbg!(header.log_type);

        let cut_data = &data[core::mem::size_of::<LogHeader>()..header.get_size() + core::mem::size_of::<LogHeader>()];
        let message = match header.log_type {
            QubicLogType::QuTransfer => {
                let log = QuTransferLog::from_bytes(&cut_data)?;

                LogMessages::QuTransferLog(log)
            },
            QubicLogType::AssetIssuance => {
                let log = AssetIssuanceLog::from_bytes(&cut_data)?;

                LogMessages::AssetIssuanceLog(log)
            },
            QubicLogType::AssetOwnershipChange => {
                let log = AssetOwnershipChangeLog::from_bytes(&cut_data)?;

                LogMessages::AssetOwnershipChangeLog(log)
            },
            QubicLogType::AssetPossessionChange => {
                let log = AssetPossessionChangeLog::from_bytes(&cut_data)?;

                LogMessages::AssetPossessionChangeLog(log)
            }
            _ => LogMessages::String(String::new())
        };

        Ok(Self {
            header,
            message
        })
    }
}