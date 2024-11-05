use super::super::super::proto_codegen::aptos::changes::{change::ChangeType, Change};
use super::super::super::proto_codegen::aptos::modules::Module;
use super::super::super::proto_codegen::aptos::resource_extras::ResourceChangeType;
use super::super::super::proto_codegen::aptos::table_items::{
    table_item::TableChangeType, TableItem,
};
use super::super::super::proto_codegen::json::resources::Resource;
use super::super::reps::{
    changes::IncompleteChangeRecord, timestamp::TimestampError,
    transaction::tx::TransactionExtraction, transaction::tx_data::TxDataExtractError,
};
use super::super::traits::{Encode, TryEncode};
use super::byte_processing::option_bytes_to_base64;
use crate::blockchain_config::processing::reps::address::AddressError;
use crate::blockchain_config::processing::reps::function::FunctionError;
use crate::blockchain_config::processing::reps::json::JsonStringError;
use crate::blockchain_config::processing::reps::module::{ModuleError, ModuleExtraction};
use crate::blockchain_config::processing::reps::moduleid::MoveModuleIdError;
use crate::blockchain_config::processing::reps::mvstruct::MvStructError;
use crate::blockchain_config::processing::reps::structtag::StructTagError;
use crate::blockchain_config::proto_codegen::aptos::modules::module::{
    ExposedFunction, Friend, Struct,
};
use aptos_protos::transaction::v1 as input_protos;

#[derive(Debug, Clone)]
pub enum ChangeError {
    Timestamp(TimestampError),
    TxData(TxDataExtractError),
    UnspecifiedType,
    UnmappedType(input_protos::write_set_change::Type),
    Address(AddressError),
    InvalidChangeTypeForRecord(ChangeType),
    MissingModuleId,
    StructTag(StructTagError),
    ModuleExtractionError(ModuleError),
    MissingModuleAbi,
    ModuleId(MoveModuleIdError),
    ExposedFunction(FunctionError),
    StructError(MvStructError),
    InvalidJsonObjStr(JsonStringError),
}

impl std::fmt::Display for ChangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ChangeError {}

impl From<JsonStringError> for ChangeError {
    fn from(value: JsonStringError) -> Self {
        ChangeError::InvalidJsonObjStr(value)
    }
}

impl From<FunctionError> for ChangeError {
    fn from(value: FunctionError) -> Self {
        Self::ExposedFunction(value)
    }
}

impl From<MvStructError> for ChangeError {
    fn from(value: MvStructError) -> Self {
        Self::StructError(value)
    }
}

impl From<StructTagError> for ChangeError {
    fn from(value: StructTagError) -> Self {
        Self::StructTag(value)
    }
}

impl From<MoveModuleIdError> for ChangeError {
    fn from(value: MoveModuleIdError) -> Self {
        Self::ModuleId(value)
    }
}

impl From<ModuleError> for ChangeError {
    fn from(value: ModuleError) -> Self {
        Self::ModuleExtractionError(value)
    }
}

impl From<AddressError> for ChangeError {
    fn from(value: AddressError) -> Self {
        Self::Address(value)
    }
}

impl From<TimestampError> for ChangeError {
    fn from(value: TimestampError) -> Self {
        Self::Timestamp(value)
    }
}
impl From<TxDataExtractError> for ChangeError {
    fn from(value: TxDataExtractError) -> Self {
        Self::TxData(value)
    }
}

pub fn convert_change_type(
    chtype: input_protos::write_set_change::Type,
) -> Result<ChangeType, ChangeError> {
    match chtype {
        input_protos::write_set_change::Type::WriteTableItem => Ok(ChangeType::WriteTableItem),
        input_protos::write_set_change::Type::DeleteTableItem => Ok(ChangeType::DeleteTableItem),
        input_protos::write_set_change::Type::WriteModule => Ok(ChangeType::WriteModule),
        input_protos::write_set_change::Type::DeleteModule => Ok(ChangeType::DeleteModule),
        input_protos::write_set_change::Type::WriteResource => Ok(ChangeType::WriteResource),
        input_protos::write_set_change::Type::DeleteResource => Ok(ChangeType::DeleteResource),
        input_protos::write_set_change::Type::Unspecified => Err(ChangeError::UnspecifiedType),
        #[allow(unreachable_patterns)]
        other => Err(ChangeError::UnmappedType(other)),
    }
}

/// ChangeRecords include records regarding changes, including records that are derived
/// from changes (resources, modules, and tableitems)
#[derive(Debug, Default)]
pub struct ChangeRecords {
    pub changes: Vec<Change>,
    pub resources: Vec<Resource>,
    pub modules: Vec<Module>,
    pub tableitems: Vec<TableItem>,
}

impl ChangeRecords {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_estimated_size(n: usize) -> Self {
        ChangeRecords {
            changes: Vec::with_capacity(n),
            resources: Vec::with_capacity(n / 3),
            modules: Vec::with_capacity(n / 3),
            tableitems: Vec::with_capacity(n / 3),
        }
    }

    pub fn len(&self) -> usize {
        self.changes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.changes.len() == 0
    }
}

pub fn get_changes(tx: &TransactionExtraction) -> Result<ChangeRecords, ChangeError> {
    // Get the changes
    let changes = &tx.tx_info.changes;
    // Create records to store the output
    let mut records = ChangeRecords::from_estimated_size(changes.len());

    let timestamp = tx.get_encoded_timestamp()?;
    let hash = tx.get_encoded_hash();
    let tx_seqnum = tx.tx_data.sequence_number()?;

    // Iterate through all the IncompleteChangeRecord
    for (index, change) in changes.iter().enumerate() {
        let chtype = convert_change_type(change.get_type())?;
        let addr = change.address().unwrap().try_encode()?;
        let state_key_hash = change.state_key_hash().unwrap().encode();
        let change_index = index as u64;

        records.changes.push(Change {
            block_height: tx.blockheight,
            block_timestamp: timestamp.clone(),
            tx_version: tx.version,
            tx_hash: hash.clone(),
            tx_sequence_number: tx.tx_data.sequence_number()?,
            change_index,
            change_type: chtype.into(),
            address: Some(addr.clone()),
            state_key_hash: state_key_hash.clone(),
            block_unixtimestamp: tx.get_unix_timestamp(),
        });

        match change {
            IncompleteChangeRecord::Module(module) => {
                let (bytecode, friends, exposed, structs) = match &module.module_data {
                    Some(data) => {
                        let bytecode = Some(data.bytecode.clone());
                        match &data.abi {
                            Some(abi) => {
                                let module_extraction = ModuleExtraction::try_from(abi)?;
                                let friends = {
                                    let mut friends: Vec<Friend> =
                                        Vec::with_capacity(module_extraction.friends.len());
                                    for friend in module_extraction.friends.iter() {
                                        friends.push(friend.try_encode()?)
                                    }
                                    friends
                                };
                                let exposed = {
                                    let mut exposed: Vec<ExposedFunction> = Vec::with_capacity(
                                        module_extraction.exposed_functions.len(),
                                    );
                                    for exposedfn in module_extraction.exposed_functions.iter() {
                                        exposed.push(exposedfn.try_encode()?);
                                    }
                                    exposed
                                };
                                let structs = {
                                    let mut structs: Vec<Struct> =
                                        Vec::with_capacity(module_extraction.structs.len());
                                    for mvstruct in module_extraction.structs.iter() {
                                        structs.push(mvstruct.try_encode()?)
                                    }
                                    structs
                                };
                                (bytecode, friends, exposed, structs)
                            }
                            None => (bytecode, Vec::new(), Vec::new(), Vec::new()),
                        }
                    }
                    None => (None::<Vec<u8>>, Vec::new(), Vec::new(), Vec::new()),
                };
                records.modules.push(Module {
                    block_height: tx.blockheight,
                    block_timestamp: timestamp.clone(),
                    tx_version: tx.version,
                    tx_hash: hash.clone(),
                    change_index,
                    bytecode: option_bytes_to_base64(&bytecode),
                    address: module.address.try_encode()?,
                    name: module
                        .module_id
                        .as_ref()
                        .map(|module_id| module_id.name.encode()),
                    friends,
                    exposed_functions: exposed,
                    structs,
                    block_unixtimestamp: tx.get_unix_timestamp(),
                })
            }
            IncompleteChangeRecord::Resource(resource) => records.resources.push(Resource {
                block_height: tx.blockheight,
                block_timestamp: timestamp.clone(),
                tx_version: tx.version,
                tx_hash: hash.clone(),
                tx_sequence_number: tx_seqnum,
                change_index,
                address: resource.address.try_encode()?,
                state_key_hash: state_key_hash.clone(),
                change_type: match chtype {
                    ChangeType::WriteResource => ResourceChangeType::WriteResource,
                    ChangeType::DeleteResource => ResourceChangeType::DeleteResource,
                    other => return Err(ChangeError::InvalidChangeTypeForRecord(other)),
                }
                .into(),
                struct_tag: match &resource.struct_tag {
                    Some(struct_tag) => Some(struct_tag.try_encode()?),
                    None => None,
                },
                type_str: resource.type_str.clone(),
                resource: resource.data.clone(),
                block_unixtimestamp: tx.get_unix_timestamp(),
            }),
            IncompleteChangeRecord::TableItem(tableitem) => records.tableitems.push(TableItem {
                block_height: tx.blockheight,
                block_timestamp: timestamp.clone(),
                tx_version: tx.version,
                tx_hash: hash.clone(),
                tx_sequence_number: tx_seqnum,
                change_index,
                address: tableitem.handle.try_encode()?,
                state_key_hash: state_key_hash.clone(),
                change_type: match chtype {
                    ChangeType::WriteTableItem => TableChangeType::WriteTableItem,
                    ChangeType::DeleteTableItem => TableChangeType::DeleteTableItem,
                    other => return Err(ChangeError::InvalidChangeTypeForRecord(other)),
                }
                .into(),
                key: tableitem.key.clone(),
                value: tableitem.value.clone(),
                block_unixtimestamp: tx.get_unix_timestamp(),
            }),
        }
    }

    Ok(records)
}
