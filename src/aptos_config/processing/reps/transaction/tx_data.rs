use super::super::address::Address;
use super::super::payload::{TxPayloadError, TxPayloadExtract};
use super::super::timestamp::{Timestamp, TimestampError};
use crate::blockchain_config::processing::{reps::address::AddressError, traits::TryEncode};
use aptos_protos::transaction::v1::{
    self as input_protos,
    transaction::{TransactionType, TxnData},
};
use log::error;

#[derive(Debug, Clone)]
pub enum TxDataExtractError {
    MissingPayload(TransactionType),
    ContextError,
    MissingUserRequest,
    MissingSignature,
    TimestampError(TimestampError),
    MissingExpirationTimestamp,
    Address(AddressError),
    GenesisPayloadError,
    MissingUserPayload,
    Payload(TxPayloadError),
}
impl From<AddressError> for TxDataExtractError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}
impl From<TxPayloadError> for TxDataExtractError {
    fn from(value: TxPayloadError) -> Self {
        Self::Payload(value)
    }
}
impl From<TimestampError> for TxDataExtractError {
    fn from(value: TimestampError) -> Self {
        Self::TimestampError(value)
    }
}

#[derive(Debug, Clone)]
pub struct TxDataExtract {
    txndata: TxnData,
}

impl From<TxnData> for TxDataExtract {
    fn from(value: TxnData) -> Self {
        TxDataExtract { txndata: value }
    }
}

impl TxDataExtract {
    /// Returns the blockmetadata
    pub fn blockmetadata(&self) -> Option<input_protos::BlockMetadataTransaction> {
        match &self.txndata {
            input_protos::transaction::TxnData::BlockMetadata(blockmetadata) => {
                Some(blockmetadata.clone())
            }
            _ => None,
        }
    }

    /// Returns the user request.
    pub fn user_request(
        &self,
    ) -> Result<Option<input_protos::UserTransactionRequest>, TxDataExtractError> {
        match &self.txndata {
            TxnData::User(userdata) => match userdata.request.clone() {
                Some(request) => Ok(Some(request)),
                None => Err(TxDataExtractError::MissingUserRequest),
            },
            _ => Ok(None),
        }
    }

    /// Returns the max gas amount
    pub fn max_gas_amount(&self) -> Result<Option<u64>, TxDataExtractError> {
        Ok(self.user_request()?.map(|user_req| user_req.max_gas_amount))
    }

    #[inline]
    pub fn sequence_number(&self) -> Result<Option<u64>, TxDataExtractError> {
        match self.user_request()? {
            Some(user_req) => Ok(Some(user_req.sequence_number)),
            None => Ok(None),
        }
    }

    #[inline]
    pub fn gas_unit_price(&self) -> Result<Option<u64>, TxDataExtractError> {
        match self.user_request()? {
            Some(user_req) => Ok(Some(user_req.gas_unit_price)),
            None => Ok(None),
        }
    }

    /// Returns the sender
    pub fn sender(&self) -> Result<Option<Address>, TxDataExtractError> {
        match self.user_request() {
            Ok(Some(req)) => Ok(Some(Address::from(&req.sender))),
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        }
    }

    /// Returns the proposer
    pub fn proposer(&self) -> Result<Option<Address>, TxDataExtractError> {
        match self.blockmetadata() {
            Some(bmd) => Ok(Some(Address::from(&bmd.proposer))),
            None => Ok(None),
        }
    }

    /// like sender(), except outputs the formatted string.
    pub fn encoded_sender(&self) -> Result<Option<String>, TxDataExtractError> {
        match self.sender() {
            Ok(Some(addr)) => Ok(Some(addr.try_encode()?)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// like proposer(), except outputs the formatted string.
    pub fn encoded_proposer(&self) -> Result<Option<String>, TxDataExtractError> {
        match self.proposer() {
            Ok(Some(addr)) => Ok(Some(addr.try_encode()?)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    /// Returns the expiration timestamp secs
    pub fn expiration_timestamp_secs(&self) -> Result<Option<Timestamp>, TxDataExtractError> {
        match self.user_request()? {
            Some(request) => match request.expiration_timestamp_secs {
                Some(expiration) => Ok(Some(expiration.try_into()?)),
                None => Err(TxDataExtractError::MissingExpirationTimestamp),
            },
            None => Ok(None),
        }
    }

    /// Returns the signature
    pub fn signature(&self) -> Result<Option<input_protos::Signature>, TxDataExtractError> {
        match self.user_request()? {
            Some(data) => match data.signature {
                Some(signature) => Ok(Some(signature)),
                None => {
                    error!("Missing signature from user request");
                    Err(TxDataExtractError::MissingSignature)
                }
            },
            None => Ok(None),
        }
    }

    /// Returns events, if the type doesn't have events, returns Ok(None).
    pub fn events(&self) -> Option<&Vec<input_protos::Event>> {
        match &self.txndata {
            TxnData::BlockMetadata(blockmetadata) => Some(&blockmetadata.events),
            TxnData::User(userdata) => Some(&userdata.events),
            TxnData::Genesis(genesisdata) => Some(&genesisdata.events),
            TxnData::Validator(validator_data) => Some(&validator_data.events),
            TxnData::StateCheckpoint(_) => None,
            TxnData::BlockEpilogue(_) => None,
        }
    }

    /// Returns the Payload.  Returns an Error if not found, return Ok(None) if not
    /// a transaction that has payloads
    pub fn payload(&self) -> Result<Option<TxPayloadExtract>, TxDataExtractError> {
        match &self.txndata {
            TxnData::User(_) => match self.user_request() {
                Ok(Some(userreq)) => {
                    match userreq.payload {
                        Some(payload) => {
                            Ok(Some(TxPayloadExtract::try_from(payload)?))
                        },
                        None => Err(TxDataExtractError::MissingUserPayload)
                    }
                },
                Err(error) => {
                    Err(error)
                }
                Ok(None) => unreachable!("Should only return Ok(None) if not userrequest, however we just verified it was."),
            }
            TxnData::Genesis(genesisdata) => {
                match &genesisdata.payload {
                    Some(genesis_payload) => match &genesis_payload.write_set {
                        Some(write_set) => Ok(Some(TxPayloadExtract::from(write_set))),
                        None => Err(TxDataExtractError::GenesisPayloadError),
                    },
                    None => Err(TxDataExtractError::GenesisPayloadError),
                }
            },
            _ => Ok(None),
        }
    }
}
