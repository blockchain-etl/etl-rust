use super::super::super::proto_codegen::aptos::blocks::Block;
use super::super::reps::transaction::tx::TransactionExtraction;
use super::byte_processing::bytes_to_base64;
use crate::blockchain_config::processing::reps::address::AddressError;
use crate::blockchain_config::processing::reps::timestamp::TimestampError;
use crate::blockchain_config::processing::reps::transaction::tx_data::TxDataExtractError;

#[derive(Debug, Clone)]
pub enum BlockError {
    Timestamp(TimestampError),
    Address(AddressError),
    TxDataExtract(TxDataExtractError),
}

impl std::fmt::Display for BlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Timestamp(err) => write!(f, "Issue handling timestamp: {}", err),
            Self::Address(err) => write!(f, "Issue handling address: {}", err),
            Self::TxDataExtract(err) => {
                write!(f, "Issue extracting data from transaction: {:?}", err)
            }
        }
    }
}

impl std::error::Error for BlockError {}

impl From<TimestampError> for BlockError {
    fn from(value: TimestampError) -> Self {
        Self::Timestamp(value)
    }
}

impl From<AddressError> for BlockError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

impl From<TxDataExtractError> for BlockError {
    fn from(value: TxDataExtractError) -> Self {
        Self::TxDataExtract(value)
    }
}

pub fn get_block(tx: &TransactionExtraction) -> Result<Option<Block>, BlockError> {
    // If not a blockmetadata, return None
    let bmd = match tx.tx_data.blockmetadata() {
        Some(bmd) => bmd,
        None => return Ok(None),
    };

    Ok(Some(Block {
        block_hash: bmd.id.clone(),
        block_timestamp: tx.get_encoded_timestamp()?,
        block_unixtimestamp: tx.get_unix_timestamp(),
        block_height: tx.blockheight,
        round: bmd.round,
        previous_block_votes_bitvec: bytes_to_base64(&bmd.previous_block_votes_bitvec),
        proposer: tx
            .tx_data
            .encoded_proposer()?
            .expect("Should be returned since blockmetadata"),
        blockmetadata_tx_version: tx.version,
    }))
}
