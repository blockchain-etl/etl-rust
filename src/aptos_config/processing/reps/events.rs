use super::super::traits::{FromVec, FromVecRef};
use super::{
    address::{Address, AddressError},
    json::JsonStringError,
    movetype::{MoveType, MoveTypeError},
};
use crate::aptos_config::proto_codegen::json::{events::Event, JsonObjectString};
use aptos_protos::transaction::v1 as input_protos;
use log::error;

#[derive(Debug, Clone)]
pub enum EventExtractionError {
    MissingEventKey,
    MissingMoveType,
    UnspecifiedMoveType,
    MoveType(MoveTypeError),
    Address(AddressError),
    JsonErr(JsonStringError),
}

impl From<AddressError> for EventExtractionError {
    #[inline]
    fn from(value: AddressError) -> Self {
        EventExtractionError::Address(value)
    }
}

impl From<MoveTypeError> for EventExtractionError {
    #[inline]
    fn from(value: MoveTypeError) -> Self {
        EventExtractionError::MoveType(value)
    }
}

#[derive(Debug, Clone)]
pub struct EventExtraction {
    pub address: Address,
    pub creation_number: u64,
    pub sequence_number: u64,
    pub r#type: MoveType,
    pub type_str: String,
    pub data: JsonObjectString,
}

impl<T> FromVec<T> for EventExtraction {}
impl<T> FromVecRef<T> for EventExtraction {}

impl TryFrom<input_protos::Event> for EventExtraction {
    type Error = EventExtractionError;
    #[inline]
    fn try_from(value: input_protos::Event) -> Result<Self, Self::Error> {
        // Extract eventkey
        let (address, creation_number) = match value.key {
            Some(eventkey) => (eventkey.account_address, eventkey.creation_number),
            None => return Err(EventExtractionError::MissingEventKey),
        };
        // Build the extracted record
        Ok(EventExtraction {
            address: (&address).into(),
            creation_number,
            sequence_number: value.sequence_number,
            r#type: match value.r#type {
                Some(movetype) => MoveType::try_from(movetype)?,
                None => return Err(EventExtractionError::MissingMoveType),
            },
            type_str: value.type_str.clone(),
            data: match JsonObjectString::try_from(value.data.clone()) {
                Ok(jsonobjstr) => jsonobjstr,
                Err(err) => {
                    panic!("Issue with JsonObjectString: {}", err);
                }
            },
        })
    }
}

use serde::ser::SerializeStruct;
// use crate::aptos_config::proto_codegen::json::JsonObjectString;

impl serde::Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Event", 12)?;
        state.serialize_field("block_height", &self.block_height)?;
        state.serialize_field("block_timestamp", &self.block_timestamp)?;
        state.serialize_field("tx_version", &self.tx_version)?;
        state.serialize_field("tx_hash", &self.tx_hash)?;
        state.serialize_field("tx_sequence_number", &self.tx_sequence_number)?;
        state.serialize_field("event_index", &self.event_index)?;
        state.serialize_field("event_type", &self.event_type)?;
        state.serialize_field("address", &self.address)?;
        state.serialize_field("creation_num", &self.creation_num)?;
        state.serialize_field("sequence_number", &self.sequence_number)?;
        match JsonObjectString::try_from(self.data.clone()) {
            Ok(jos) => state.serialize_field("data", &jos)?,
            Err(err) => {
                error!("Failed to build j son object string: {}", err);
                return Err(serde::ser::Error::custom(err));
            }
        }
        state.serialize_field("block_unixtimestamp", &self.block_unixtimestamp)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EventVisitor;

        impl<'de> serde::de::Visitor<'de> for EventVisitor {
            type Value = Event;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("an Event struct")
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
                let mut event_index = None;
                let mut event_type = None;
                let mut address = None;
                let mut creation_num = None;
                let mut sequence_number = None;
                let mut data = None;
                let mut block_unixtimestamp = None;

                while let Some(key) = map.next_key::<::prost::alloc::string::String>()? {
                    match key.as_str() {
                        "block_height" => block_height = Some(map.next_value()?),
                        "block_timestamp" => block_timestamp = Some(map.next_value()?),
                        "tx_version" => tx_version = Some(map.next_value()?),
                        "tx_hash" => tx_hash = Some(map.next_value()?),
                        "tx_sequence_number" => tx_sequence_number = Some(map.next_value()?),
                        "event_index" => event_index = Some(map.next_value()?),
                        "event_type" => event_type = Some(map.next_value()?),
                        "address" => address = Some(map.next_value()?),
                        "creation_num" => creation_num = Some(map.next_value()?),
                        "sequence_number" => sequence_number = Some(map.next_value()?),
                        "data" => data = Some(map.next_value()?),
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
                let event_index =
                    event_index.ok_or_else(|| serde::de::Error::missing_field("event_index"))?;
                let event_type =
                    event_type.ok_or_else(|| serde::de::Error::missing_field("event_type"))?;
                let address = address.ok_or_else(|| serde::de::Error::missing_field("address"))?;
                let creation_num =
                    creation_num.ok_or_else(|| serde::de::Error::missing_field("creation_num"))?;
                let sequence_number = sequence_number
                    .ok_or_else(|| serde::de::Error::missing_field("sequence_number"))?;
                let data = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let block_unixtimestamp = block_unixtimestamp
                    .ok_or_else(|| serde::de::Error::missing_field("block_unixtimestamp"))?;

                Ok(Event {
                    block_height,
                    block_timestamp,
                    tx_version,
                    tx_hash,
                    tx_sequence_number,
                    event_index,
                    event_type,
                    address,
                    creation_num,
                    sequence_number,
                    data,
                    block_unixtimestamp,
                })
            }
        }

        deserializer.deserialize_struct(
            "Event",
            &[
                "block_height",
                "block_timestamp",
                "tx_version",
                "tx_hash",
                "tx_sequence_number",
                "event_index",
                "event_type",
                "address",
                "creation_num",
                "sequence_number",
                "data",
                "block_unixtimestamp",
            ],
            EventVisitor,
        )
    }
}
