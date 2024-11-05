use super::super::traits::{FromVec, FromVecRef, TryEncode};
use super::abilties::AbilityError;
use super::structtag::{StructTag, StructTagError};
use aptos_protos::transaction::v1 as input_protos;

/// The prefix which we want to remove from the movetype.
const MOVETYPE_PREFIX: &str = "MOVE_TYPES_";

#[derive(Debug, Clone)]
pub enum MoveTypeError {
    UnspecifiedMoveTypeError,
    MoveTypeDecodeError(prost::DecodeError, i32),
    LackedPrefix(String),
    MissingContent(input_protos::MoveTypes),
    Unparsable(String),
    UnsupportedAdvMoveType(input_protos::MoveTypes),
    EmptyReference,
    ReferenceTypeError(Box<MoveTypeError>),
    Ability(Box<AbilityError>),
    ContextError(input_protos::MoveTypes),
    StructTag(StructTagError),
    MismatchContent,
}

impl std::fmt::Display for MoveTypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnspecifiedMoveTypeError => write!(f, ""),
            Self::MoveTypeDecodeError(err, num) => {
                write!(f, "Failed to decode MoveType from {}: {}", num, err)
            }
            Self::LackedPrefix(string) => write!(
                f,
                "Failed to strip prefix from MoveType, verify string: {}",
                string
            ),
            Self::MissingContent(mvtype) => write!(
                f,
                "Additional content missing from MoveType typical with content: {:?}",
                mvtype
            ),
            Self::Unparsable(string) => write!(f, "MoveType unparsable: \"{}\"", string),
            Self::UnsupportedAdvMoveType(mvtype) => write!(
                f,
                "MoveType has additional content we are not prepared to handle: {:?}",
                mvtype
            ),
            Self::EmptyReference => write!(f, "Reference content is missing the type it points to"),
            Self::ReferenceTypeError(err) => write!(f, "Issue with reference movetype: {}", err),
            Self::Ability(err) => write!(f, "Issue with the ability: {}", err),
            Self::ContextError(mvtype) => write!(f, "MoveType used in wrong context: {:?}", mvtype),
            Self::StructTag(err) => write!(f, "Failed to encode StructTag: {}", err),
            Self::MismatchContent => write!(f, "Content was not what was expected."),
        }
    }
}

impl From<AbilityError> for MoveTypeError {
    fn from(value: AbilityError) -> Self {
        MoveTypeError::Ability(Box::new(value))
    }
}

impl From<StructTagError> for MoveTypeError {
    fn from(value: StructTagError) -> Self {
        MoveTypeError::StructTag(value)
    }
}

/// Represents a MoveType
#[derive(Debug, Clone)]
pub struct MoveType {
    /// The movetype
    movetype: input_protos::MoveTypes,
    /// Additional content
    content: Option<input_protos::move_type::Content>,
}

impl MoveType {
    pub fn new(
        movetype: &input_protos::MoveTypes,
        content: &Option<input_protos::move_type::Content>,
    ) -> Self {
        Self {
            movetype: *movetype,
            content: content.clone(),
        }
    }

    /// Returns true if MoveType is advanced and contains
    /// additional details (content)
    pub fn has_additional_content(&self) -> bool {
        self.content.is_some()
    }

    /// Returns the MoveType
    pub fn movetype(&self) -> &input_protos::MoveTypes {
        &self.movetype
    }

    /// Returns as tuple
    pub fn to_tuple(
        &self,
    ) -> (
        input_protos::MoveTypes,
        Option<input_protos::move_type::Content>,
    ) {
        (*self.movetype(), self.content.clone())
    }
}

impl TryFrom<input_protos::MoveType> for MoveType {
    type Error = MoveTypeError;

    fn try_from(value: input_protos::MoveType) -> Result<Self, Self::Error> {
        Ok(Self {
            movetype: match input_protos::MoveTypes::try_from(value.r#type) {
                Ok(mvtype) => mvtype,
                Err(error) => return Err(MoveTypeError::MoveTypeDecodeError(error, value.r#type)),
            },
            content: value.content.clone(),
        })
    }
}

impl TryFrom<&input_protos::MoveType> for MoveType {
    type Error = MoveTypeError;

    fn try_from(value: &input_protos::MoveType) -> Result<Self, Self::Error> {
        Ok(Self {
            movetype: match input_protos::MoveTypes::try_from(value.r#type) {
                Ok(mvtype) => mvtype,
                Err(error) => return Err(MoveTypeError::MoveTypeDecodeError(error, value.r#type)),
            },
            content: value.content.clone(),
        })
    }
}

impl TryEncode<String> for MoveType {
    type Error = MoveTypeError;

    fn try_encode(&self) -> Result<String, Self::Error> {
        // Detect if it is an advanced type (contains content)
        if !self.has_additional_content() {
            match self.movetype() {
                // If type matches one of the known advanced types,
                // we need to raise a MissingContent error
                input_protos::MoveTypes::Reference
                | input_protos::MoveTypes::GenericTypeParam
                | input_protos::MoveTypes::Struct
                | input_protos::MoveTypes::Vector
                | input_protos::MoveTypes::Unparsable => {
                    return Err(MoveTypeError::MissingContent(*self.movetype()))
                }
                // Try to strip the prefix.  If it didn't we, need to examine
                // the specific case
                other => {
                    let mtype_str = other.as_str_name();
                    match mtype_str.strip_prefix(MOVETYPE_PREFIX) {
                        Some(newstring) => Ok(String::from(newstring)),
                        None => Err(MoveTypeError::LackedPrefix(String::from(mtype_str))),
                    }
                }
            }
        } else {
            match (self.movetype(), &self.content) {
                (
                    input_protos::MoveTypes::Reference,
                    Some(input_protos::move_type::Content::Reference(reference)),
                ) => {
                    let prefix = match reference.mutable {
                        true => "&mut ",
                        false => "&",
                    };

                    let type_str = match &reference.to {
                        Some(mvtype) => match MoveType::try_from((**mvtype).clone()) {
                            Ok(nested_mvtype) => match nested_mvtype.try_encode() {
                                Ok(string) => string,
                                Err(mverr) => {
                                    return Err(MoveTypeError::ReferenceTypeError(Box::new(mverr)))
                                }
                            },
                            Err(err) => {
                                return Err(MoveTypeError::ReferenceTypeError(Box::new(err)))
                            }
                        },
                        None => return Err(MoveTypeError::EmptyReference),
                    };
                    Ok(format!("{}{}", prefix, type_str))
                }
                (
                    input_protos::MoveTypes::Vector,
                    Some(input_protos::move_type::Content::Vector(inner)),
                ) => Ok(format!(
                    "Vector<{}>",
                    Self::try_from((**inner).clone())?.try_encode()?
                )),
                (
                    input_protos::MoveTypes::Struct,
                    Some(input_protos::move_type::Content::Struct(tag)),
                ) => Ok(StructTag::try_from(tag.clone())?.try_encode()?),
                (
                    input_protos::MoveTypes::GenericTypeParam,
                    Some(input_protos::move_type::Content::GenericTypeParamIndex(index)),
                ) => Ok(format!("T{}", index)), //Ok(Ability::try_from(*index+1)?.try_encode()?),
                (
                    input_protos::MoveTypes::Unparsable,
                    Some(input_protos::move_type::Content::Unparsable(badstring)),
                ) => Err(MoveTypeError::Unparsable(badstring.clone())),
                (mvtype, _) => Err(MoveTypeError::UnsupportedAdvMoveType(*mvtype)),
            }
        }
    }
}

impl<T> FromVec<T> for MoveType {}
impl<T> FromVecRef<T> for MoveType {}
