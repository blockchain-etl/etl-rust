// use crate::blockchain_config::proto_codegen::transactions::transaction::payload::code::Abi;
use crate::blockchain_config::processing::traits::Encode;
use crate::blockchain_config::proto_codegen::aptos::modules::module::Struct;
use aptos_protos::transaction::v1 as input_protos;

use super::super::traits::{FromVec, FromVecRef, TryEncode};
use super::abilties::{Ability, AbilityError};
use super::generictypeparam::{GenericTypeParam, GenericTypeParamError};
use super::movetype::{MoveType, MoveTypeError};
use super::structtag::StructName;

#[derive(Debug, Clone)]
pub enum MvStructError {
    MissingFieldType(String),
    MoveType(MoveTypeError),
    StructFieldMissingMoveType,
    Ability(AbilityError),
    GenericTypeParam(GenericTypeParamError),
}

impl From<AbilityError> for MvStructError {
    fn from(value: AbilityError) -> Self {
        Self::Ability(value)
    }
}

impl From<GenericTypeParamError> for MvStructError {
    fn from(value: GenericTypeParamError) -> Self {
        Self::GenericTypeParam(value)
    }
}

impl From<MoveTypeError> for MvStructError {
    fn from(value: MoveTypeError) -> Self {
        Self::MoveType(value)
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub mvtype: MoveType,
}

impl TryFrom<&input_protos::MoveStructField> for StructField {
    type Error = MvStructError;
    fn try_from(value: &input_protos::MoveStructField) -> Result<Self, Self::Error> {
        Ok(StructField {
            name: value.name.clone(),
            mvtype: match value.r#type.clone() {
                Some(mvtype) => MoveType::try_from(mvtype)?,
                None => return Err(MvStructError::StructFieldMissingMoveType),
            },
        })
    }
}

use super::super::super::proto_codegen::aptos::modules::module::r#struct::Fields;

impl TryEncode<Fields> for StructField {
    type Error = MvStructError;
    fn try_encode(&self) -> Result<Fields, Self::Error> {
        Ok(Fields {
            name: self.name.clone(),
            r#type: self.mvtype.try_encode()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MvStruct {
    pub name: StructName,
    pub is_native: bool,
    pub abilities: Vec<Ability>,
    pub generic_type_params: Vec<GenericTypeParam>,
    pub fields: Vec<StructField>,
}

impl<T> FromVec<T> for MvStruct {}
impl<T> FromVecRef<T> for MvStruct {}

impl TryFrom<input_protos::MoveStruct> for MvStruct {
    type Error = MvStructError;
    fn try_from(value: input_protos::MoveStruct) -> Result<Self, Self::Error> {
        Ok(Self {
            name: StructName::from(&value.name),
            is_native: value.is_native,
            abilities: Vec::from_iter(value.abilities().map(Ability::from)),
            generic_type_params: Vec::from_iter(
                value.generic_type_params.iter().map(GenericTypeParam::from),
            ),
            fields: {
                let mut fields = Vec::with_capacity(value.fields.len());
                for field in &value.fields {
                    fields.push(StructField::try_from(field)?);
                }
                fields
            },
        })
    }
}

impl TryFrom<&input_protos::MoveStruct> for MvStruct {
    type Error = MvStructError;
    fn try_from(value: &input_protos::MoveStruct) -> Result<Self, Self::Error> {
        Ok(Self {
            name: StructName::from(&value.name),
            is_native: value.is_native,
            abilities: Vec::from_iter(value.abilities().map(Ability::from)),
            generic_type_params: Vec::from_iter(
                value.generic_type_params.iter().map(GenericTypeParam::from),
            ),
            fields: {
                let mut fields = Vec::with_capacity(value.fields.len());
                for field in &value.fields {
                    fields.push(StructField::try_from(field)?);
                }
                fields
            },
        })
    }
}

impl TryEncode<Struct> for MvStruct {
    type Error = MvStructError;
    fn try_encode(&self) -> Result<Struct, Self::Error> {
        Ok(Struct {
            name: self.name.encode(),
            is_native: self.is_native,
            abilities: {
                let mut abilities = Vec::with_capacity(self.abilities.len());
                for ability in self.abilities.iter() {
                    abilities.push(ability.try_encode()?)
                }
                abilities
            },
            generic_type_params: {
                let mut gtps = Vec::with_capacity(self.generic_type_params.len());
                for gtp in self.generic_type_params.iter() {
                    gtps.push(gtp.try_encode()?);
                }
                gtps
            },
            fields: {
                let mut fields = Vec::with_capacity(self.fields.len());
                for field in self.fields.iter() {
                    fields.push(field.try_encode()?)
                }
                fields
            },
        })
    }
}
