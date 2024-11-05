use aptos_protos::transaction::v1::{
    transaction_payload::Payload as TxPayloadData, transaction_payload::Type as TxPayloadType,
    write_set::WriteSet, ScriptPayload, TransactionPayload,
};
// use crate::blockchain_config::processing::traits::FromVecRef;
use super::super::super::proto_codegen::aptos::transactions::transaction::{
    payload::Code, Payload, PayloadType,
};
use super::super::tables::byte_processing::bytes_to_base64;
use super::super::traits::{FromVecRef, TryEncode};
use super::function::{Function, FunctionError};
use super::moduleid::{ModuleId, MoveModuleIdError};
use super::movetype::{MoveType, MoveTypeError};
use log::{info, warn};
#[derive(Debug, Clone)]
pub enum TxPayloadError {
    UnspecifiedPayloadType,
    MismatchPayloadData(TxPayloadType),
    MissingPayloadData(TxPayloadType),
    DeprecatedModuleBundlePayload,
    FunctionMissingModuleId,
    EntryFnMissingFnId,
    ModuleId(MoveModuleIdError),
    MoveType(MoveTypeError),
    ScriptPayloadMissingCode,
    FailedBuildingMoveScriptAbi(FunctionError),
    MoveScriptMissingAbi,
    EmptyWritesetPayload,
    NoScriptPayloadInWriteSetPayload,
    UnspecifiedWritesetType,
    MismatchWriteset,
    NotWriteSet,
    MissingWritesetData,
    MissingPayloadType,
}

impl From<MoveTypeError> for TxPayloadError {
    fn from(value: MoveTypeError) -> Self {
        Self::MoveType(value)
    }
}

impl From<MoveModuleIdError> for TxPayloadError {
    fn from(value: MoveModuleIdError) -> Self {
        Self::ModuleId(value)
    }
}

#[derive(Debug, Clone)]
pub struct TxPayloadExtract {
    pub payload_type: Option<TxPayloadType>,
    pub payload_data: Option<TxPayloadData>,
    pub genesis_payload: Option<WriteSet>,
}

impl TxPayloadExtract {
    pub fn encode_functionid(&self) -> Result<Option<String>, TxPayloadError> {
        match &self.payload_data {
            Some(TxPayloadData::EntryFunctionPayload(data)) => match &data.function {
                Some(fnid) => match &fnid.module {
                    Some(mid) => {
                        let moveid: String = ModuleId::from(mid).try_encode()?;
                        Ok(Some(format!("{}::{}", moveid, fnid.name)))
                    }
                    None => Err(TxPayloadError::FunctionMissingModuleId),
                },
                None => Err(TxPayloadError::FunctionMissingModuleId),
            },
            _ => Ok(None),
        }
    }

    #[inline]
    pub fn is_genesis(&self) -> bool {
        self.genesis_payload.is_some()
    }

    #[inline]
    pub fn is_genesis_scriptwriteset(&self) -> bool {
        match &self.genesis_payload {
            Some(writeset) => match &writeset {
                WriteSet::ScriptWriteSet(_) => true,
                WriteSet::DirectWriteSet(_) => false,
            },
            None => false,
        }
    }

    pub fn encode_payload_type(&self) -> Result<String, TxPayloadError> {
        match self.payload_type {
            Some(TxPayloadType::EntryFunctionPayload) => Ok(PayloadType::EntryFunction.into()),
            Some(TxPayloadType::MultisigPayload) => Ok(PayloadType::Multisig.into()),
            Some(TxPayloadType::ScriptPayload) => Ok(PayloadType::Script.into()),
            Some(TxPayloadType::WriteSetPayload) => Ok(PayloadType::Writeset.into()),
            // Outdated, from old protos
            // Some(TxPayloadType::ModuleBundlePayload) => Ok(PayloadType::ModuleBundle.into()),
            Some(TxPayloadType::Unspecified) => Err(TxPayloadError::UnspecifiedPayloadType),
            None => {
                if self.is_genesis() {
                    Ok(PayloadType::GenesisWriteset.into())
                } else {
                    Err(TxPayloadError::MissingPayloadType)
                }
            }
        }
    }

    pub fn type_arguments(&self) -> Result<Option<Vec<String>>, TxPayloadError> {
        let args = match &self.payload_data {
            Some(TxPayloadData::EntryFunctionPayload(data)) => &data.type_arguments,
            Some(TxPayloadData::ScriptPayload(data)) => &data.type_arguments,
            Some(TxPayloadData::WriteSetPayload(_)) => {
                &self.extract_writesetpayload_scriptpayload()?.type_arguments
            }
            _ => {
                if self.is_genesis_scriptwriteset() {
                    &self.extract_writesetpayload_scriptpayload()?.type_arguments
                } else {
                    return Ok(None);
                }
            }
        };
        // Translate into our MoveTypes
        let movetypes = MoveType::try_from_vecref(args)?;

        let encoded_args: Vec<String> = {
            let mut encoded_args = Vec::with_capacity(args.len());
            for mt in movetypes.iter() {
                encoded_args.push(mt.try_encode()?)
            }
            encoded_args
        };
        Ok(Some(encoded_args))
    }

    pub fn arguments(&self) -> Result<Option<Vec<String>>, TxPayloadError> {
        match &self.payload_data {
            Some(TxPayloadData::EntryFunctionPayload(data)) => Ok(Some(data.arguments.clone())),
            Some(TxPayloadData::ScriptPayload(data)) => Ok(Some(data.arguments.clone())),
            Some(TxPayloadData::WriteSetPayload(_)) => Ok(Some(
                self.extract_writesetpayload_scriptpayload()?
                    .arguments
                    .clone(),
            )),
            _ => {
                if self.is_genesis_scriptwriteset() {
                    Ok(Some(
                        self.extract_writesetpayload_scriptpayload()?
                            .arguments
                            .clone(),
                    ))
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub fn entry_function_id_str(&self) -> Option<String> {
        match &self.payload_data {
            Some(TxPayloadData::EntryFunctionPayload(data)) => {
                Some(data.entry_function_id_str.clone())
            }
            _ => None,
        }
    }

    fn extract_writesetpayload(&self) -> Result<&WriteSet, TxPayloadError> {
        match &self.payload_data {
            Some(TxPayloadData::WriteSetPayload(writesetpayload)) => {
                match &writesetpayload.write_set {
                    Some(outter_write_set) => match &outter_write_set.write_set {
                        Some(write_set) => Ok(write_set),
                        None => Err(TxPayloadError::MissingWritesetData),
                    },
                    None => Err(TxPayloadError::NotWriteSet),
                }
            }
            None => match &self.genesis_payload {
                Some(writeset) => Ok(writeset),
                None => Err(TxPayloadError::MissingWritesetData),
            },
            _ => Err(TxPayloadError::NotWriteSet),
        }
    }

    fn extract_writesetpayload_scriptpayload(&self) -> Result<&ScriptPayload, TxPayloadError> {
        match self.extract_writesetpayload()? {
            WriteSet::ScriptWriteSet(sws) => {
                match &sws.script {
                    Some(script) => Ok(script),
                    None => {
                        warn!("ScriptPayloadMissingCode err from `extract_writesetpayload_scriptpayload`");
                        Err(TxPayloadError::ScriptPayloadMissingCode)
                    }
                }
            }
            _ => Err(TxPayloadError::NoScriptPayloadInWriteSetPayload),
        }
    }

    pub fn code(&self) -> Result<Option<Code>, TxPayloadError> {
        let mmbytecode = match &self.payload_data {
            Some(TxPayloadData::ScriptPayload(data)) => match &data.code {
                Some(mmbytecode) => mmbytecode,
                None => {
                    warn!("ScriptPayloadMissingCode err from `code` w/ ScriptPayload");
                    return Err(TxPayloadError::ScriptPayloadMissingCode);
                }
            },
            Some(TxPayloadData::WriteSetPayload(_)) => {
                match &self.extract_writesetpayload_scriptpayload()?.code {
                    Some(code) => code,
                    None => {
                        warn!("ScriptPayloadMissingCode err from `code` w/ WriteSetPayload");
                        return Err(TxPayloadError::ScriptPayloadMissingCode);
                    }
                }
            }
            _ => {
                if self.is_genesis_scriptwriteset() {
                    match &self.extract_writesetpayload_scriptpayload()?.code {
                        Some(code) => code,
                        None => {
                            warn!("ScriptPayloadMissingCode err from `code` w/ genesis_scriptwriteset");
                            return Err(TxPayloadError::ScriptPayloadMissingCode);
                        }
                    }
                } else {
                    return Ok(None);
                }
            }
        };
        let code = Code {
            bytecode: bytes_to_base64(&mmbytecode.bytecode),
            abi: match &mmbytecode.abi {
                Some(fxn) => match Function::try_from(fxn) {
                    Ok(abi) => match abi.try_into() {
                        Ok(abi_fnrep) => Some(abi_fnrep),
                        Err(error) => {
                            return Err(TxPayloadError::FailedBuildingMoveScriptAbi(error))
                        }
                    },
                    Err(err) => return Err(TxPayloadError::FailedBuildingMoveScriptAbi(err)),
                },
                None => {
                    info!("`code` w/ missing abi");
                    None
                }
            },
        };

        Ok(Some(code))
    }

    pub fn multisig_address(&self) -> Option<String> {
        match &self.payload_data {
            Some(TxPayloadData::MultisigPayload(data)) => Some(data.multisig_address.clone()),
            _ => None,
        }
    }

    pub fn execute_as(&self) -> Result<Option<String>, TxPayloadError> {
        match &self.payload_data {
            Some(TxPayloadData::WriteSetPayload(_)) => match self.extract_writesetpayload()? {
                WriteSet::ScriptWriteSet(sws) => Ok(Some(sws.execute_as.clone())),
                _ => Err(TxPayloadError::MissingWritesetData),
            },
            None => match &self.genesis_payload {
                Some(writeset) => match writeset {
                    WriteSet::DirectWriteSet(_) => Ok(None),
                    WriteSet::ScriptWriteSet(sws) => Ok(Some(sws.execute_as.clone())),
                },
                None => Ok(None),
            },
            _ => Ok(None),
        }
    }
}

impl TryEncode<Payload> for TxPayloadExtract {
    type Error = TxPayloadError;
    fn try_encode(&self) -> Result<Payload, Self::Error> {
        Ok(Payload {
            function: self.encode_functionid()?,
            type_arguments: self.type_arguments()?.unwrap_or(Vec::new()),
            arguments: self.arguments()?.unwrap_or(Vec::new()),
            entry_function_id_str: self.entry_function_id_str(),
            code: self.code()?,
            multisig_address: self.multisig_address(),
            execute_as: self.execute_as()?,
        })
    }
}

impl From<&WriteSet> for TxPayloadExtract {
    fn from(value: &WriteSet) -> Self {
        TxPayloadExtract {
            payload_type: None,
            payload_data: None,
            genesis_payload: Some(value.clone()),
        }
    }
}

impl TryFrom<TransactionPayload> for TxPayloadExtract {
    type Error = TxPayloadError;
    fn try_from(value: TransactionPayload) -> Result<Self, Self::Error> {
        match (value.r#type(), value.payload) {
            (
                txtype @ TxPayloadType::EntryFunctionPayload,
                Some(txdata @ TxPayloadData::EntryFunctionPayload(_)),
            ) => Ok(TxPayloadExtract {
                payload_type: Some(txtype),
                payload_data: Some(txdata),
                genesis_payload: None,
            }),
            // Deprecated variant, can re-add if necessary
            // (TxPayloadType::ModuleBundlePayload, Some(TxPayloadData::ModuleBundlePayload(_))) => {
            //     Err(TxPayloadError::DeprecatedModuleBundlePayload)
            // }
            (
                txtype @ TxPayloadType::MultisigPayload,
                Some(txdata @ TxPayloadData::MultisigPayload(_)),
            ) => Ok(TxPayloadExtract {
                payload_type: Some(txtype),
                payload_data: Some(txdata),
                genesis_payload: None,
            }),
            (
                txtype @ TxPayloadType::ScriptPayload,
                Some(txdata @ TxPayloadData::ScriptPayload(_)),
            ) => Ok(TxPayloadExtract {
                payload_type: Some(txtype),
                payload_data: Some(txdata),
                genesis_payload: None,
            }),
            (
                txtype @ TxPayloadType::WriteSetPayload,
                Some(txdata @ TxPayloadData::WriteSetPayload(_)),
            ) => Ok(TxPayloadExtract {
                payload_type: Some(txtype),
                payload_data: Some(txdata),
                genesis_payload: None,
            }),
            (TxPayloadType::Unspecified, _) => Err(TxPayloadError::UnspecifiedPayloadType),
            (payload_type, None) => Err(TxPayloadError::MissingPayloadData(payload_type)),
            (payload_type, Some(_)) => Err(TxPayloadError::MismatchPayloadData(payload_type)),
        }
    }
}
