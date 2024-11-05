use super::super::super::proto_codegen::aptos::transactions::Transaction;
use super::super::reps::address::AddressError;
use super::super::reps::timestamp::TimestampError;
use super::super::reps::transaction::tx::TransactionExtraction;
use crate::blockchain_config::processing::reps::payload::TxPayloadError;
use crate::blockchain_config::processing::reps::transaction::tx::TxExtractionError;
use crate::blockchain_config::processing::reps::transaction::tx_data::TxDataExtractError;
use crate::blockchain_config::processing::reps::transaction::tx_info::TxInfoExtractionError;
use crate::blockchain_config::processing::traits::{Encode, TryEncode};
use crate::blockchain_config::proto_codegen::aptos::transactions::transaction::TxType;
use log::debug;

#[derive(Debug, Clone)]
pub enum TxError {
    Timestamp(TimestampError),
    TxDataExtract(TxDataExtractError),
    TxInfoExtract(TxInfoExtractionError),
    Address(AddressError),
    MissingSender,
    Payload(TxPayloadError),
    TxExtractError(TxExtractionError),
    MissingSigCount(TxType),
}
impl std::fmt::Display for TxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for TxError {}

impl From<TxExtractionError> for TxError {
    fn from(value: TxExtractionError) -> Self {
        Self::TxExtractError(value)
    }
}
impl From<TxPayloadError> for TxError {
    fn from(value: TxPayloadError) -> Self {
        Self::Payload(value)
    }
}
impl From<TxDataExtractError> for TxError {
    fn from(value: TxDataExtractError) -> Self {
        Self::TxDataExtract(value)
    }
}
impl From<TxInfoExtractionError> for TxError {
    fn from(value: TxInfoExtractionError) -> Self {
        Self::TxInfoExtract(value)
    }
}
impl From<TimestampError> for TxError {
    fn from(value: TimestampError) -> Self {
        Self::Timestamp(value)
    }
}
impl From<AddressError> for TxError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

pub fn get_transactions(
    tx: &TransactionExtraction,
    sig_cnt: Option<u64>,
) -> Result<Transaction, TxError> {
    match tx.tx_info.success {
        true => get_successful_transaction(tx, sig_cnt),
        false => get_failed_transaction(tx, sig_cnt),
    }
}

pub fn get_successful_transaction(
    tx: &TransactionExtraction,
    sig_cnt: Option<u64>,
) -> Result<Transaction, TxError> {
    let (payload, payload_type) = {
        match tx.tx_data.payload()? {
            None => (None, None),
            Some(extract) => (
                Some(extract.try_encode()?),
                Some(extract.encode_payload_type()?),
            ),
        }
    };

    Ok(Transaction {
        block_height: tx.blockheight,
        block_timestamp: tx.get_encoded_timestamp()?,
        tx_type: tx.get_txtype()?.into(),
        tx_version: tx.version,
        tx_hash: tx.get_encoded_hash(),
        state_change_hash: tx.tx_info.state_change_hash.encode(),
        event_root_hash: tx.tx_info.event_root_hash.encode(),
        state_checkpoint_hash: tx
            .tx_info
            .state_checkpoint_hash
            .as_ref()
            .map(|item| item.encode()),
        gas_used: tx.tx_info.gas_used,
        success: tx.tx_info.success,
        vm_status: Some(tx.tx_info.vm_status.clone()),
        accumulator_root_hash: Some(tx.tx_info.accumulator_root_hash.encode()),
        sequence_number: tx.tx_data.sequence_number()?,
        max_gas_amount: tx.tx_data.max_gas_amount()?,
        sender: match tx.tx_data.sender()? {
            Some(addr) => Some(addr.try_encode()?),
            None => {
                if tx.is_usertx() {
                    return Err(TxError::MissingSender);
                } else {
                    None
                }
            }
        },
        num_changes: tx.tx_info.calculate_changes_aggregate()?,
        num_events: tx.tx_data.events().map(|events| events.len() as u64),
        // expiration timestamp is a special case since we expect user input, even if
        // we get a timestamp back, we may not be able to convert it into the
        // expected formatting due to OutOfRangeUtc.  In this case, we will leave it
        // blank still.
        expiration_timestamp: match tx.tx_data.expiration_timestamp_secs() {
            Ok(Some(timestamp)) => match timestamp.try_encode() {
                Ok(string) => Some(string),
                Err(TimestampError::OutOfRangeUtc(s, n)) => {
                    debug!("Out of range for UTC: {} seconds, {} nanos", s, n);
                    None
                }
                Err(TimestampError::OutOfRangeBigQuery(s, n)) => {
                    debug!("Out of range for BQ: {} seconds, {} nanos", s, n);
                    None
                }
                Err(error) => return Err(error.into()),
            },
            Ok(None) => None,
            Err(err) => return Err(err.into()),
        },
        payload,
        payload_type,
        gas_unit_price: tx.tx_data.gas_unit_price()?,
        block_unixtimestamp: tx.get_unix_timestamp(),
        // Note: unlike expiration_timestamp, we should alays be able to place the
        // the unixtimestamp
        expiration_unixtimestamp: match tx.tx_data.expiration_timestamp_secs() {
            Ok(None) => None,
            Ok(Some(timestamp)) => Some(timestamp.encode()),
            Err(err) => return Err(err.into()),
        },
        num_signatures: match (sig_cnt, tx.get_txtype()) {
            // We only expect signature values for Users
            (Some(sig_val), Ok(TxType::User)) => Some(sig_val),
            // We are missing the signature counts
            (None, Ok(TxType::User)) => return Err(TxError::MissingSigCount(TxType::User)),
            (_, Err(err)) => return Err(TxError::TxExtractError(err)),
            (_, _) => None,
        },
    })
}

pub fn get_failed_transaction(
    tx: &TransactionExtraction,
    sig_cnt: Option<u64>,
) -> Result<Transaction, TxError> {
    let (payload, payload_type) = {
        match tx.tx_data.payload() {
            Ok(None) => (None, None),
            Ok(Some(extract)) => {
                let payload = match extract.try_encode() {
                    Ok(payload) => Some(payload),
                    Err(_) => None,
                };
                let payload_type = match extract.encode_payload_type() {
                    Ok(payload_type) => Some(payload_type),
                    Err(_) => None,
                };
                (payload, payload_type)
            }
            Err(_) => (None, None),
        }
    };

    Ok(Transaction {
        block_height: tx.blockheight,
        block_timestamp: tx.get_encoded_timestamp()?,
        tx_type: tx.get_txtype()?.into(),
        tx_version: tx.version,
        tx_hash: tx.get_encoded_hash(),
        state_change_hash: tx.tx_info.state_change_hash.encode(),
        event_root_hash: tx.tx_info.event_root_hash.encode(),
        state_checkpoint_hash: tx
            .tx_info
            .state_checkpoint_hash
            .as_ref()
            .map(|item| item.encode()),
        gas_used: tx.tx_info.gas_used,
        success: tx.tx_info.success,
        vm_status: Some(tx.tx_info.vm_status.clone()),
        accumulator_root_hash: Some(tx.tx_info.accumulator_root_hash.encode()),
        sequence_number: tx.tx_data.sequence_number().unwrap_or(None),
        max_gas_amount: tx.tx_data.max_gas_amount().unwrap_or_default(),
        sender: match tx.tx_data.sender() {
            Ok(Some(addr)) => match addr.try_encode() {
                Ok(encoded_addr) => Some(encoded_addr),
                Err(_) => None,
            },
            Ok(None) => {
                if tx.is_usertx() {
                    return Err(TxError::MissingSender);
                } else {
                    None
                }
            }
            Err(_) => None,
        },
        num_changes: tx.tx_info.calculate_changes_aggregate()?,
        num_events: tx.tx_data.events().map(|events| events.len() as u64),
        // expiration timestamp is a special case since we expect user input, even if
        // we get a timestamp back, we may not be able to convert it into the
        // expected formatting due to OutOfRangeUtc.  In this case, we will leave it
        // blank still.
        expiration_timestamp: match tx.tx_data.expiration_timestamp_secs() {
            Ok(Some(timestamp)) => match timestamp.try_encode() {
                Ok(string) => Some(string),
                Err(TimestampError::OutOfRangeUtc(s, n)) => {
                    debug!("Out of range for UTC: {} seconds, {} nanos", s, n);
                    Some(String::from(
                        super::super::reps::timestamp::BIGQUERRY_MAX_TIMESTAMP_STRING,
                    ))
                }
                Err(TimestampError::OutOfRangeBigQuery(s, n)) => {
                    debug!("Out of range for BQ: {} seconds, {} nanos", s, n);
                    Some(String::from(
                        super::super::reps::timestamp::BIGQUERRY_MAX_TIMESTAMP_STRING,
                    ))
                }
                Err(error) => return Err(error.into()),
            },
            Ok(None) => None,
            Err(err) => return Err(err.into()),
        },
        payload,
        payload_type,
        gas_unit_price: match tx.tx_data.gas_unit_price() {
            Ok(Some(gup)) => Some(gup),
            Ok(None) => None,
            Err(_) => None,
        },
        block_unixtimestamp: tx.get_unix_timestamp(),
        // Note: unlike expiration_timestamp, we should alays be able to place the
        // the unixtimestamp
        expiration_unixtimestamp: match tx.tx_data.expiration_timestamp_secs() {
            Ok(None) => None,
            Ok(Some(timestamp)) => Some(timestamp.encode()),
            Err(_) => None,
        },
        num_signatures: sig_cnt,
    })
}
