use super::super::super::proto_codegen::aptos::signatures::Signature;
use super::super::reps::address::AddressError;
use super::super::reps::signatures::{SignatureError, SignatureSubRecord};
use super::super::reps::transaction::{tx::TransactionExtraction, tx_data::TxDataExtractError};
use super::super::traits::TryEncode;
use crate::blockchain_config::processing::deferred::DeferredError;
use crate::blockchain_config::processing::reps::timestamp::TimestampError;

#[derive(Debug, Clone)]
pub enum SigProcessError {
    TxDataExtract(TxDataExtractError),
    SubRecordFailure(SignatureError),
    Timestamp(TimestampError),
    DeferredError,
    Address(AddressError),
}
impl std::fmt::Display for SigProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for SigProcessError {}

impl From<AddressError> for SigProcessError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}
impl From<TxDataExtractError> for SigProcessError {
    fn from(value: TxDataExtractError) -> Self {
        Self::TxDataExtract(value)
    }
}
impl From<SignatureError> for SigProcessError {
    fn from(value: SignatureError) -> Self {
        Self::SubRecordFailure(value)
    }
}
impl From<TimestampError> for SigProcessError {
    fn from(value: TimestampError) -> Self {
        Self::Timestamp(value)
    }
}
impl<T> From<DeferredError<T>> for SigProcessError {
    fn from(_: DeferredError<T>) -> Self {
        Self::DeferredError
    }
}

pub fn get_signatures(
    tx: &TransactionExtraction,
) -> Result<Option<Vec<Signature>>, SigProcessError> {
    // create subrecords.  These are partial records potentially with
    // incomplete data built from the signature data alone.
    let subrecords = match tx.tx_data.signature()? {
        // Go ahead and return Ok(None) if no signature
        None => return Ok(None),
        Some(signature) => SignatureSubRecord::from_signature(signature)?,
    };
    // Create output
    let mut records = Vec::with_capacity(subrecords.len());

    let timestamp: String = tx.get_encoded_timestamp()?;
    let hash: String = tx.get_encoded_hash();
    let default_signer = tx.tx_data.encoded_sender()?.expect("Should be usertx");

    for subrecord in subrecords {
        records.push(Signature {
            block_height: tx.blockheight,
            block_timestamp: timestamp.clone(),
            tx_version: tx.version,
            tx_hash: hash.clone(),
            threshold: subrecord.threshold,
            is_secondary: subrecord.is_secondary.extract()?,
            is_fee_payer: subrecord.is_feepayer.extract()?,
            is_sender: subrecord.is_sender.extract()?,
            signature: subrecord.signature,
            public_key: subrecord.public_key,
            build_type: subrecord.buildtype.extract()?.into(),
            signer: match subrecord.signer.extract() {
                Ok(addr) => addr.try_encode()?,
                Err(_) => default_signer.clone(),
            },
            block_unixtimestamp: tx.get_unix_timestamp(),
        })
    }
    Ok(Some(records))
}
