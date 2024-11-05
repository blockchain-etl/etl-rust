use crate::blockchain_config::proto_codegen::json::resources::Resource;
use crate::blockchain_config::proto_codegen::json::JsonObjectString;

use super::super::super::proto_codegen::aptos::table_items::table_item::{
    Key as TableKey, Value as TableValue,
};
use super::address::Address;
use super::hashval::HashValue;
use super::moduleid::ModuleId;
use super::structtag::{StructTag, StructTagError};
use aptos_protos::transaction::v1::{
    self as input_protos, write_set_change::Change as ChangeData,
    write_set_change::Type as ChangeType,
};
use log::error;

#[derive(Debug, Clone)]
pub enum IncompleteChangeRecord {
    Resource(IncompleteResourceRecord),
    TableItem(IncompleteTableItemRecord),
    Module(IncompleteModuleRecord),
}

impl IncompleteChangeRecord {
    pub fn get_type(&self) -> input_protos::write_set_change::Type {
        match self {
            Self::Module(module) => module.change_type,
            Self::TableItem(tableitem) => tableitem.change_type,
            Self::Resource(resource) => resource.change_type,
        }
    }

    pub fn address(&self) -> Option<Address> {
        match self {
            Self::Module(module) => Some(module.address.clone()),
            Self::Resource(resource) => Some(resource.address.clone()),
            Self::TableItem(tableitem) => Some(tableitem.handle.clone()),
        }
    }

    pub fn state_key_hash(&self) -> Option<HashValue> {
        match self {
            Self::Module(module) => Some(module.state_key_hash.clone()),
            Self::Resource(resource) => Some(resource.state_key_hash.clone()),
            Self::TableItem(tableitem) => Some(tableitem.state_key_hash.clone()),
        }
    }
}

impl TryFrom<input_protos::WriteSetChange> for IncompleteChangeRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::WriteSetChange) -> Result<Self, Self::Error> {
        match (value.r#type(), value.change) {
            (ChangeType::DeleteModule, Some(ChangeData::DeleteModule(data))) => {
                Ok(IncompleteChangeRecord::Module(data.try_into()?))
            }
            (ChangeType::WriteModule, Some(ChangeData::WriteModule(data))) => {
                Ok(IncompleteChangeRecord::Module(data.try_into()?))
            }
            (ChangeType::DeleteResource, Some(ChangeData::DeleteResource(data))) => {
                Ok(IncompleteChangeRecord::Resource(data.try_into()?))
            }
            (ChangeType::WriteResource, Some(ChangeData::WriteResource(data))) => {
                Ok(IncompleteChangeRecord::Resource(data.try_into()?))
            }
            (ChangeType::WriteTableItem, Some(ChangeData::WriteTableItem(data))) => {
                Ok(IncompleteChangeRecord::TableItem(data.try_into()?))
            }
            (ChangeType::DeleteTableItem, Some(ChangeData::DeleteTableItem(data))) => {
                Ok(IncompleteChangeRecord::TableItem(data.try_into()?))
            }
            (_, Some(_)) => Err(ChangeError::MismatchChangeData),
            (_, None) => Err(ChangeError::MissingChangeData),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ChangeError {
    MissingChangeData,
    MissingModuleBytecodeData,
    MissingModuleId,
    MissingTableData,
    MismatchChangeData,
    StructTagError(StructTagError),
}

impl std::fmt::Display for ChangeError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingChangeData => write!(f, "Missing critical Change Data"),
            Self::MissingModuleBytecodeData => write!(f, "Missing module bytecode data"),
            Self::MissingModuleId => write!(f, "Missing Module Id"),
            Self::MissingTableData => write!(f, "Missing TableData"),
            Self::MismatchChangeData => write!(f, "Unsure how to handle data"),
            Self::StructTagError(err) => write!(f, "Failed to interpret struct tag: {}", err),
        }
    }
}

impl From<StructTagError> for ChangeError {
    fn from(value: StructTagError) -> Self {
        Self::StructTagError(value)
    }
}

impl std::error::Error for ChangeError {}

#[derive(Debug, Clone)]
pub struct IncompleteResourceRecord {
    pub change_type: input_protos::write_set_change::Type,
    pub address: Address,
    pub state_key_hash: HashValue,
    pub struct_tag: Option<StructTag>,
    pub type_str: String,
    pub data: Option<String>,
}

impl IncompleteResourceRecord {
    #[inline]
    pub fn is_writeresource(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::WriteResource
        )
    }
    #[inline]
    pub fn is_deleteresource(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::DeleteResource
        )
    }
}

impl TryFrom<input_protos::DeleteResource> for IncompleteResourceRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::DeleteResource) -> Result<Self, Self::Error> {
        Ok(IncompleteResourceRecord {
            change_type: input_protos::write_set_change::Type::DeleteResource,
            address: (&value.address).into(),
            state_key_hash: value.state_key_hash.into(),
            struct_tag: match value.r#type {
                Some(tag) => Some(StructTag::try_from(tag)?),
                None => None,
            },
            type_str: value.type_str.clone(),
            data: None,
        })
    }
}

impl TryFrom<input_protos::WriteResource> for IncompleteResourceRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::WriteResource) -> Result<Self, Self::Error> {
        Ok(IncompleteResourceRecord {
            change_type: input_protos::write_set_change::Type::WriteResource,
            address: (&value.address).into(),
            state_key_hash: value.state_key_hash.into(),
            type_str: value.type_str.clone(),
            data: Some(value.data.clone()),
            struct_tag: match value.r#type {
                Some(tag) => Some(StructTag::try_from(tag)?),
                None => None,
            },
        })
    }
}

use serde::ser::SerializeStruct;

impl serde::Serialize for Resource {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Resource", 13)?;

        state.serialize_field("block_height", &self.block_height)?;
        state.serialize_field("block_timestamp", &self.block_timestamp)?;
        state.serialize_field("tx_version", &self.tx_version)?;
        state.serialize_field("tx_hash", &self.tx_hash)?;
        state.serialize_field("tx_sequence_number", &self.tx_sequence_number)?;
        state.serialize_field("change_index", &self.change_index)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("state_key_hash", &self.state_key_hash)?;
        state.serialize_field("change_type", &self.change_type)?;
        state.serialize_field("struct_tag", &self.struct_tag)?;
        state.serialize_field("type_str", &self.type_str)?;

        match &self.resource {
            Some(resource) => match JsonObjectString::try_from(resource.clone()) {
                Ok(jos) => state.serialize_field("resource", &jos)?,
                Err(err) => {
                    error!("Failed to serialize JsonObjectString: {}", err);
                    return Err(serde::ser::Error::custom(err));
                }
            },
            None => state.serialize_field("resource", &None::<String>)?,
        }

        state.serialize_field("block_unixtimestamp", &self.block_unixtimestamp)?;

        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Resource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ResourceVisitor;

        impl<'de> serde::de::Visitor<'de> for ResourceVisitor {
            type Value = Resource;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Resource struct")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut block_height = None;
                let mut block_timestamp = None;
                let mut tx_version = None;
                let mut tx_hash = None;
                let mut tx_sequence_number = None;
                let mut change_index = None;
                let mut address = None;
                let mut state_key_hash = None;
                let mut change_type = None;
                let mut struct_tag = None;
                let mut type_str = None;
                let mut resource = None;
                let mut block_unixtimestamp = None;

                while let Some(key) = map.next_key::<::prost::alloc::string::String>()? {
                    match key.as_str() {
                        "block_height" => block_height = Some(map.next_value()?),
                        "block_timestamp" => block_timestamp = Some(map.next_value()?),
                        "tx_version" => tx_version = Some(map.next_value()?),
                        "tx_hash" => tx_hash = Some(map.next_value()?),
                        "tx_sequence_number" => tx_sequence_number = Some(map.next_value()?),
                        "change_index" => change_index = Some(map.next_value()?),
                        "address" => address = Some(map.next_value()?),
                        "state_key_hash" => state_key_hash = Some(map.next_value()?),
                        "change_type" => change_type = Some(map.next_value()?),
                        "struct_tag" => struct_tag = Some(map.next_value()?),
                        "type_str" => type_str = Some(map.next_value()?),
                        "resource" => resource = Some(map.next_value()?),
                        "block_unixtimestamp" => block_unixtimestamp = Some(map.next_value()?),
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let block_height =
                    block_height.ok_or_else(|| serde::de::Error::missing_field("block_height"))?;
                let block_timestamp = block_timestamp
                    .ok_or_else(|| serde::de::Error::missing_field("block_timestamp"))?;
                let tx_version =
                    tx_version.ok_or_else(|| serde::de::Error::missing_field("tx_version"))?;
                let tx_hash = tx_hash.ok_or_else(|| serde::de::Error::missing_field("tx_hash"))?;
                let change_index =
                    change_index.ok_or_else(|| serde::de::Error::missing_field("change_index"))?;
                let address = address.ok_or_else(|| serde::de::Error::missing_field("address"))?;
                let state_key_hash = state_key_hash
                    .ok_or_else(|| serde::de::Error::missing_field("state_key_hash"))?;
                let change_type =
                    change_type.ok_or_else(|| serde::de::Error::missing_field("change_type"))?;
                let type_str =
                    type_str.ok_or_else(|| serde::de::Error::missing_field("type_str"))?;
                let block_unixtimestamp = block_unixtimestamp
                    .ok_or_else(|| serde::de::Error::missing_field("block_unixtimestamp"))?;

                Ok(Resource {
                    block_height,
                    block_timestamp,
                    tx_version,
                    tx_hash,
                    tx_sequence_number,
                    change_index,
                    address,
                    state_key_hash,
                    change_type,
                    struct_tag,
                    type_str,
                    resource,
                    block_unixtimestamp,
                })
            }
        }

        deserializer.deserialize_struct(
            "Resource",
            &[
                "block_height",
                "block_timestamp",
                "tx_version",
                "tx_hash",
                "tx_sequence_number",
                "change_index",
                "address",
                "state_key_hash",
                "change_type",
                "struct_tag",
                "type_str",
                "resource",
                "block_unixtimestamp",
            ],
            ResourceVisitor,
        )
    }
}

#[derive(Debug, Clone)]
pub struct IncompleteTableItemRecord {
    pub change_type: input_protos::write_set_change::Type,
    pub state_key_hash: HashValue,
    pub handle: Address,
    pub key: TableKey,
    pub value: Option<TableValue>,
}

impl IncompleteTableItemRecord {
    #[inline]
    pub fn is_writetableitem(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::WriteTableItem
        )
    }
    #[inline]
    pub fn is_deletetableitem(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::DeleteTableItem
        )
    }
}

/// Strip the quotation marks
#[inline]
fn strip_quotes(string: &str) -> String {
    let string_noprefix = string.strip_prefix('\"').unwrap_or(string);
    String::from(string_noprefix.strip_suffix('\"').unwrap_or(string))
}

impl TryFrom<input_protos::WriteTableItem> for IncompleteTableItemRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::WriteTableItem) -> Result<Self, Self::Error> {
        let (key, val) = match value.data {
            Some(tabledata) => (
                TableKey {
                    name: strip_quotes(&tabledata.key),
                    r#type: tabledata.key_type.clone(),
                },
                Some(TableValue {
                    content: strip_quotes(&tabledata.value),
                    r#type: tabledata.value_type.clone(),
                }),
            ),
            None => return Err(ChangeError::MissingTableData),
        };

        Ok(IncompleteTableItemRecord {
            change_type: input_protos::write_set_change::Type::WriteTableItem,
            state_key_hash: value.state_key_hash.into(),
            handle: (&value.handle).into(),
            key,
            value: val,
        })
    }
}

impl TryFrom<input_protos::DeleteTableItem> for IncompleteTableItemRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::DeleteTableItem) -> Result<Self, Self::Error> {
        Ok(IncompleteTableItemRecord {
            change_type: input_protos::write_set_change::Type::DeleteTableItem,
            state_key_hash: value.state_key_hash.into(),
            handle: (&value.handle).into(),
            key: match value.data {
                Some(tabledata) => TableKey {
                    name: strip_quotes(&tabledata.key),
                    r#type: tabledata.key_type.clone(),
                },
                None => return Err(ChangeError::MissingTableData),
            },
            value: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct IncompleteModuleRecord {
    pub change_type: input_protos::write_set_change::Type,
    pub address: Address,
    pub state_key_hash: HashValue,
    pub module_id: Option<ModuleId>,
    pub module_data: Option<input_protos::MoveModuleBytecode>,
}

impl IncompleteModuleRecord {
    #[inline]
    pub fn is_writemodule(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::WriteModule
        )
    }
    #[inline]
    pub fn is_deletemodule(&self) -> bool {
        matches!(
            self.change_type,
            input_protos::write_set_change::Type::DeleteModule
        )
    }
}

impl TryFrom<input_protos::WriteModule> for IncompleteModuleRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::WriteModule) -> Result<Self, ChangeError> {
        // Keep bytecode mandatory for now
        let module_data = match value.data {
            Some(bytecodedata) => Some(bytecodedata),
            None => return Err(ChangeError::MissingModuleBytecodeData),
        };
        // Allow Module Id to be established.
        let module_id = match &module_data {
            Some(mmbc) => mmbc
                .abi
                .as_ref()
                .map(|abi| ModuleId::new(&abi.address, abi.name.clone())),
            None => None,
        };

        Ok(Self {
            change_type: input_protos::write_set_change::Type::WriteModule,
            address: (&value.address).into(),
            state_key_hash: value.state_key_hash.into(),
            module_id,
            module_data,
        })
    }
}

impl TryFrom<input_protos::DeleteModule> for IncompleteModuleRecord {
    type Error = ChangeError;
    #[inline]
    fn try_from(value: input_protos::DeleteModule) -> Result<Self, ChangeError> {
        Ok(Self {
            change_type: input_protos::write_set_change::Type::DeleteModule,
            address: (&value.address).into(),
            state_key_hash: value.state_key_hash.into(),
            module_id: value.module.map(|module_id| (&module_id).into()),
            module_data: None,
        })
    }
}
