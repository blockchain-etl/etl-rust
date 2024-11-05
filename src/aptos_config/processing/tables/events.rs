use crate::blockchain_config::processing::reps::transaction::tx_data::TxDataExtractError;
use crate::blockchain_config::processing::traits::TryEncode;

use super::super::super::proto_codegen::json::events::Event;
use super::super::reps::address::AddressError;
use super::super::reps::events::{EventExtraction, EventExtractionError};
use super::super::reps::timestamp::TimestampError;
use super::super::reps::transaction::tx::TransactionExtraction;

#[derive(Debug, Clone)]
pub enum EventError {
    EventExtraction(EventExtractionError),
    Timestamp(TimestampError),
    Address(AddressError),
    TxDataExtract(TxDataExtractError),
}
impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EventError {}

impl From<TimestampError> for EventError {
    fn from(value: TimestampError) -> Self {
        Self::Timestamp(value)
    }
}
impl From<EventExtractionError> for EventError {
    fn from(value: EventExtractionError) -> Self {
        Self::EventExtraction(value)
    }
}
impl From<AddressError> for EventError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}
impl From<TxDataExtractError> for EventError {
    fn from(value: TxDataExtractError) -> Self {
        Self::TxDataExtract(value)
    }
}

/// Returns event records
pub fn get_events(tx: &TransactionExtraction) -> Result<Option<Vec<Event>>, EventError> {
    match tx.tx_data.events() {
        Some(events) => {
            let mut records: Vec<Event> = Vec::with_capacity(events.len());
            let timestamp = tx.get_encoded_timestamp()?;
            let hash = tx.get_encoded_hash();
            for (index, event) in events.iter().enumerate() {
                let extraction = EventExtraction::try_from(event.clone())?;

                records.push(Event {
                    block_height: tx.blockheight,
                    block_timestamp: timestamp.clone(),
                    tx_version: tx.version,
                    tx_hash: hash.clone(),
                    tx_sequence_number: tx.tx_data.sequence_number()?,
                    event_index: index as u64,
                    address: extraction.address.try_encode()?,
                    event_type: extraction.type_str.clone(),
                    creation_num: extraction.creation_number,
                    sequence_number: extraction.sequence_number,
                    data: extraction.data.clone().to_string(),
                    block_unixtimestamp: tx.get_unix_timestamp(),
                })
            }
            Ok(Some(records))
        }
        None => Ok(None),
    }
}
