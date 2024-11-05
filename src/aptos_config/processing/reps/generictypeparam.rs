use crate::blockchain_config::processing::traits::{FromVec, FromVecRef, TryEncode};
use crate::blockchain_config::proto_codegen::aptos::modules::module::MoveAbility;

use super::super::super::proto_codegen::aptos::modules::module::{
    exposed_function::GenericFunctionTypeParams, r#struct::GenericStructTypeParams,
};
use super::abilties::{Ability, AbilityError};
use aptos_protos::transaction::v1::{
    self as input_protos, MoveFunctionGenericTypeParam, MoveStructGenericTypeParam,
};
use log::error;

#[derive(Debug, Clone)]
pub enum GenericTypeParamError {
    FunctionHasNoPhantom,
    Ability(AbilityError),
    IncorrectGenericType,
}

impl std::fmt::Display for GenericTypeParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FunctionHasNoPhantom => write!(
                f,
                "GenericTypeParams for Functions do not have `is_phantom` value."
            ),
            Self::Ability(err) => write!(f, "Issue with Ability: {}", err),
            Self::IncorrectGenericType => write!(f, "Issue with incorrect generic type"),
        }
    }
}

impl From<AbilityError> for GenericTypeParamError {
    fn from(value: AbilityError) -> Self {
        GenericTypeParamError::Ability(value)
    }
}

#[derive(Debug, Clone)]
pub enum GenericTypeParam {
    Struct(Vec<Ability>, bool),
    Function(Vec<Ability>),
}

impl TryEncode<GenericFunctionTypeParams> for GenericTypeParam {
    type Error = GenericTypeParamError;
    fn try_encode(&self) -> Result<GenericFunctionTypeParams, Self::Error> {
        match self {
            Self::Function(abilities) => {
                let mut constraints: Vec<MoveAbility> = Vec::with_capacity(abilities.len());
                for ability in abilities {
                    constraints.push(match ability.try_encode() {
                        Ok(ability) => ability,
                        Err(err) => {
                            println!("Failed while encoding GenericFunctionTypeParams");
                            return Err(err.into());
                        }
                    })
                }
                let constraint_numbers: Vec<_> =
                    Vec::from_iter(constraints.iter().map(|a| (*a).into()));

                Ok(GenericFunctionTypeParams {
                    constraints: constraint_numbers,
                })
            }
            _ => Err(GenericTypeParamError::IncorrectGenericType),
        }
    }
}

impl TryEncode<GenericStructTypeParams> for GenericTypeParam {
    type Error = GenericTypeParamError;
    fn try_encode(&self) -> Result<GenericStructTypeParams, Self::Error> {
        match self {
            Self::Struct(abilities, is_phantom) => {
                let mut constraints: Vec<MoveAbility> = Vec::with_capacity(abilities.len());
                for ability in abilities {
                    constraints.push(match ability.try_encode() {
                        Ok(ability) => ability,
                        Err(err) => {
                            println!("Failed while encoding GenericStructTypeParams");
                            return Err(err.into());
                        }
                    })
                }
                let constraint_numbers: Vec<_> =
                    Vec::from_iter(constraints.iter().map(|a| (*a).into()));

                Ok(GenericStructTypeParams {
                    constraints: constraint_numbers,
                    is_phantom: *is_phantom,
                })
            }
            _ => Err(GenericTypeParamError::IncorrectGenericType),
        }
    }
}

impl From<&MoveFunctionGenericTypeParam> for GenericTypeParam {
    fn from(value: &MoveFunctionGenericTypeParam) -> Self {
        let constraints = Vec::from_iter(value.constraints().map(Ability::from));
        Self::new(&constraints, None)
    }
}
impl From<MoveFunctionGenericTypeParam> for GenericTypeParam {
    fn from(value: MoveFunctionGenericTypeParam) -> Self {
        let constraints = Vec::from_iter(value.constraints().map(Ability::from));
        Self::new(&constraints, None)
    }
}
impl From<&MoveStructGenericTypeParam> for GenericTypeParam {
    fn from(value: &MoveStructGenericTypeParam) -> Self {
        let constraints = Vec::from_iter(value.constraints().map(Ability::from));
        Self::new(&constraints, Some(value.is_phantom))
    }
}
impl From<MoveStructGenericTypeParam> for GenericTypeParam {
    fn from(value: MoveStructGenericTypeParam) -> Self {
        let constraints = Vec::from_iter(value.constraints().map(Ability::from));
        Self::new(&constraints, Some(value.is_phantom))
    }
}

impl TryFrom<Vec<input_protos::MoveType>> for GenericTypeParam {
    type Error = GenericTypeParamError;
    #[inline]
    fn try_from(value: Vec<input_protos::MoveType>) -> Result<Self, Self::Error> {
        match value
            .iter()
            .map(|move_type| Ability::try_from(move_type.clone()))
            .collect::<Result<Vec<Ability>, AbilityError>>()
        {
            Ok(abilities) => Ok(Self::from(abilities)),
            Err(ability_err) => {
                error!("Failed converting MoveType to GenericTypeParam");
                Err(ability_err.into())
            }
        }
    }
}

impl<T> From<Vec<T>> for GenericTypeParam
where
    T: Into<Ability> + Clone,
{
    fn from(value: Vec<T>) -> Self {
        let constraints: Vec<Ability> = Vec::from_iter(value.iter().map(|a| a.clone().into()));
        Self::new(&constraints, None)
    }
}

impl<T> FromVec<T> for GenericTypeParam {}
impl<T> FromVecRef<T> for GenericTypeParam {}

impl<T> From<&Vec<T>> for GenericTypeParam
where
    T: Into<Ability> + Clone,
{
    fn from(value: &Vec<T>) -> Self {
        let constraints: Vec<Ability> = Vec::from_iter(value.iter().map(|a| a.clone().into()));
        Self::new(&constraints, None)
    }
}

// impl From<Vec<Ability>> for GenericTypeParam {
//     fn from(value: Vec<Ability>) -> Self {
//         GenericTypeParam::Function(value.clone())
//     }
// }

impl GenericTypeParam {
    /// Creates a new GenericTypeParam
    #[inline]
    pub fn new(constraints: &[Ability], is_phantom: Option<bool>) -> Self {
        match is_phantom {
            Some(is_phantom) => Self::Struct(constraints.to_vec(), is_phantom),
            None => Self::Function(constraints.to_vec()),
        }
    }

    /// Returns true if set for structs
    #[inline]
    pub fn is_for_struct(&self) -> bool {
        matches!(self, Self::Struct(_, _))
    }
    /// Returns true if set for functions
    #[inline]
    pub fn is_for_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    /// Returns the boolean of the is_phantom value.  Returns an
    /// error if it is not for Structs
    #[inline]
    pub fn is_phantom(&self) -> Result<bool, GenericTypeParamError> {
        match self {
            Self::Struct(_, is_phantom) => Ok(*is_phantom),
            Self::Function(_) => Err(GenericTypeParamError::FunctionHasNoPhantom),
        }
    }

    /// Returns the constraints
    pub fn constraints(&self) -> &Vec<Ability> {
        match self {
            Self::Function(constraints) => constraints,
            Self::Struct(constraints, _) => constraints,
        }
    }

    /// Returns the constraints as a singular string, with given separator and optionally
    /// provided items to enclose the results (i.e. with < and >).  Returns Ok(None) if
    /// no items are given.
    pub fn constraint_string(
        &self,
        sep: &str,
        enclosed_with: Option<(&str, &str)>,
        prefix: Option<&str>,
    ) -> Result<Option<String>, GenericTypeParamError> {
        let constraints = self.constraints();

        if constraints.is_empty() {
            return Ok(None);
        }

        let strings: Result<Vec<String>, GenericTypeParamError> = constraints
            .iter()
            .map(|a| a.try_encode().map_err(GenericTypeParamError::from))
            .collect();

        let result_string = strings?.join(sep);

        let (left, right) = enclosed_with.unwrap_or(("", ""));

        Ok(Some(format!(
            "{}{}{}{}",
            prefix.unwrap_or(""),
            left,
            result_string,
            right
        )))
    }
}
