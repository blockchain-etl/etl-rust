use super::super::super::super::proto_codegen::aptos::{
    common::UnixTimestamp, transactions::transaction::TxType,
};
use super::super::super::traits::{Encode, TryEncode};
use super::super::timestamp::{Timestamp, TimestampEncodedType, TimestampError};
use super::{
    tx_data::{TxDataExtract, TxDataExtractError},
    tx_info::{TransactionInfoExtraction, TxInfoExtractionError},
};
use aptos_protos::transaction::v1::{
    self as input_protos,
    transaction::{TransactionType, TxnData},
};

#[derive(Debug, Clone)]
pub enum TxExtractionError {
    MissingTimeStamp,
    TimestampError(TimestampError),
    MissingInfo,
    MissingData,
    InvalidData,
    Unspecified,
    TxInfoExtractionError(TxInfoExtractionError),
    TxDataExtractionError(TxDataExtractError),
}

impl std::fmt::Display for TxExtractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TxExtractionError {}

impl From<TimestampError> for TxExtractionError {
    fn from(value: TimestampError) -> Self {
        TxExtractionError::TimestampError(value)
    }
}
impl From<TxInfoExtractionError> for TxExtractionError {
    fn from(value: TxInfoExtractionError) -> Self {
        TxExtractionError::TxInfoExtractionError(value)
    }
}
impl From<TxDataExtractError> for TxExtractionError {
    fn from(value: TxDataExtractError) -> Self {
        TxExtractionError::TxDataExtractionError(value)
    }
}

#[derive(Debug, Clone)]
pub struct TransactionExtraction {
    pub version: u64,
    pub epoch: u64,
    pub blockheight: i64,
    pub timestamp: Timestamp,
    pub tx_type: TransactionType,
    pub tx_info: TransactionInfoExtraction,
    pub tx_data: TxDataExtract,
}

impl TransactionExtraction {
    /// Returns true if it is a user transaction
    #[inline]
    pub fn is_usertx(&self) -> bool {
        matches!(self.tx_type, TransactionType::User)
    }
    /// Returns true if it is a blockmetadata transaction
    #[inline]
    pub fn is_blockmetadata(&self) -> bool {
        matches!(self.tx_type, TransactionType::BlockMetadata)
    }

    /// Returns the timestamp in the encoded format
    #[inline]
    pub fn get_encoded_timestamp(&self) -> Result<TimestampEncodedType, TimestampError> {
        self.timestamp.try_encode()
    }

    /// Returns the unix timestamp
    #[inline]
    pub fn get_unix_timestamp(&self) -> UnixTimestamp {
        self.timestamp.encode()
    }

    #[inline]
    pub fn get_encoded_hash(&self) -> String {
        self.tx_info.hash.encode()
    }

    #[inline]
    pub fn convert_txtype(intype: TransactionType) -> Result<TxType, TxExtractionError> {
        match intype {
            TransactionType::BlockMetadata => Ok(TxType::BlockMetadata),
            TransactionType::StateCheckpoint => Ok(TxType::StateCheckpoint),
            TransactionType::User => Ok(TxType::User),
            TransactionType::Genesis => Ok(TxType::Genesis),
            TransactionType::Validator => Ok(TxType::Validator),
            TransactionType::BlockEpilogue => Ok(TxType::BlockEpilogue),
            TransactionType::Unspecified => Err(TxExtractionError::Unspecified),
        }
    }

    #[inline]
    pub fn get_txtype(&self) -> Result<TxType, TxExtractionError> {
        Self::convert_txtype(self.tx_type)
    }
}

impl TryFrom<input_protos::Transaction> for TransactionExtraction {
    type Error = TxExtractionError;

    fn try_from(value: input_protos::Transaction) -> Result<Self, Self::Error> {
        // Extract timestamp
        let timestamp = match &value.timestamp {
            Some(input_timestamp) => Timestamp::try_from(input_timestamp.clone())?,
            None => return Err(TxExtractionError::MissingTimeStamp),
        };
        // Extract the type and data. Validate that they match, and that it's not unspecified.
        let (tx_type, tx_data) = match (value.r#type(), &value.txn_data) {
            (TransactionType::Unspecified, _) => return Err(TxExtractionError::Unspecified),
            (tx_type @ TransactionType::User, Some(tx_data @ TxnData::User(_))) => {
                (tx_type, tx_data)
            }
            (tx_type @ TransactionType::Genesis, Some(tx_data @ TxnData::Genesis(_))) => {
                (tx_type, tx_data)
            }
            (
                tx_type @ TransactionType::BlockMetadata,
                Some(tx_data @ TxnData::BlockMetadata(_)),
            ) => (tx_type, tx_data),
            (
                tx_type @ TransactionType::StateCheckpoint,
                Some(tx_data @ TxnData::StateCheckpoint(_)),
            ) => (tx_type, tx_data),
            (tx_type @ TransactionType::Validator, Some(tx_data @ TxnData::Validator(_))) => {
                (tx_type, tx_data)
            }
            (_, None) => return Err(TxExtractionError::MissingData),
            (_, Some(_)) => return Err(TxExtractionError::InvalidData),
        };
        // Extract the TxnInfo
        let tx_info = match &value.info {
            Some(tx_info) => tx_info,
            None => return Err(TxExtractionError::MissingInfo),
        };
        // Place into a TransactionSubRecord
        Ok(TransactionExtraction {
            version: value.version,
            epoch: value.epoch,
            blockheight: value.block_height as i64,
            timestamp,
            tx_type,
            tx_info: TransactionInfoExtraction::try_from(tx_info.clone())?,
            tx_data: TxDataExtract::from(tx_data.clone()),
        })
    }
}
