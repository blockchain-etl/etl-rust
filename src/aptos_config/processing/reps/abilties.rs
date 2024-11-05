use super::super::super::proto_codegen::aptos::modules::module::MoveAbility;
use super::{
    super::traits::{FromVec, FromVecRef, TryEncode},
    movetype::MoveTypeError,
};
use aptos_protos::transaction::v1 as input_protos;

const ABILITY_PREFIX: &str = "MOVE_ABILITY_";

#[derive(Debug, Clone)]
pub enum AbilityError {
    Unspecified,
    DecodeError(prost::DecodeError, i32),
    MissingPrefix(String),
    MoveType(MoveTypeError),
    MoveTypeContext,
}

impl std::fmt::Display for AbilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DecodeError(err, num) => {
                write!(f, "Failed to decode Ability from number {}: {}", num, err)
            }
            Self::Unspecified => write!(f, "Received an unspecified ability"),
            Self::MissingPrefix(string) => {
                write!(f, "Missing prefix, may need to investigate: \"{}\"", string)
            }
            Self::MoveType(err) => write!(f, "Issue with MoveType: {}", err),
            Self::MoveTypeContext => write!(f, "Incorrect context of MoveType"),
        }
    }
}

impl From<MoveTypeError> for AbilityError {
    fn from(value: MoveTypeError) -> Self {
        AbilityError::MoveType(value)
    }
}

impl std::error::Error for AbilityError {}

#[derive(Debug, Clone)]
pub struct Ability {
    inner: input_protos::MoveAbility,
}

impl<T> FromVec<T> for Ability {}
impl<T> FromVecRef<T> for Ability {}

impl From<input_protos::MoveAbility> for Ability {
    fn from(value: input_protos::MoveAbility) -> Self {
        Self { inner: value }
    }
}

impl From<&input_protos::MoveAbility> for Ability {
    fn from(value: &input_protos::MoveAbility) -> Self {
        Self { inner: *value }
    }
}

impl TryFrom<i32> for Ability {
    type Error = AbilityError;
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match input_protos::MoveAbility::try_from(value) {
            Ok(ability) => Ok(Self::from(ability)),
            Err(error) => Err(AbilityError::DecodeError(error, value)),
        }
    }
}

impl TryFrom<u32> for Ability {
    type Error = AbilityError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Self::try_from(value as i32)
    }
}

impl TryFrom<input_protos::MoveType> for Ability {
    type Error = AbilityError;
    fn try_from(value: input_protos::MoveType) -> Result<Self, Self::Error> {
        match value.r#type() {
            input_protos::MoveTypes::GenericTypeParam => match value.content {
                Some(input_protos::move_type::Content::GenericTypeParamIndex(index)) => {
                    Ok(Self::try_from(index)?)
                }
                Some(_) => Err(MoveTypeError::MismatchContent.into()),
                None => Err(MoveTypeError::MissingContent(
                    input_protos::MoveTypes::GenericTypeParam,
                )
                .into()),
            },
            input_protos::MoveTypes::Unspecified => Err(AbilityError::Unspecified),
            input_protos::MoveTypes::Unparsable => match value.content {
                Some(input_protos::move_type::Content::Unparsable(badstring)) => {
                    Err(MoveTypeError::Unparsable(badstring).into())
                }
                None => Err(MoveTypeError::Unparsable(String::from(
                    "UNABLE TO GET UNPARSABLE STRING",
                ))
                .into()),
                Some(_) => Err(MoveTypeError::Unparsable(String::from("UNKNOWN CONTENT")).into()),
            },
            _ => Err(AbilityError::MoveTypeContext),
        }
    }
}

impl TryEncode<String> for Ability {
    type Error = AbilityError;
    fn try_encode(&self) -> Result<String, Self::Error> {
        match self.inner {
            input_protos::MoveAbility::Unspecified => Err(AbilityError::Unspecified),
            other => {
                let ability_str = other.as_str_name();
                match ability_str.strip_prefix(ABILITY_PREFIX) {
                    Some(ability_str) => Ok(String::from(ability_str)),
                    None => Err(AbilityError::MissingPrefix(String::from(ability_str))),
                }
            }
        }
    }
}

impl TryEncode<MoveAbility> for Ability {
    type Error = AbilityError;

    fn try_encode(&self) -> Result<MoveAbility, Self::Error> {
        match self.inner {
            input_protos::MoveAbility::Copy => Ok(MoveAbility::Copy),
            input_protos::MoveAbility::Drop => Ok(MoveAbility::Drop),
            input_protos::MoveAbility::Key => Ok(MoveAbility::Key),
            input_protos::MoveAbility::Store => Ok(MoveAbility::Store),
            input_protos::MoveAbility::Unspecified => Err(AbilityError::Unspecified),
        }
    }
}
