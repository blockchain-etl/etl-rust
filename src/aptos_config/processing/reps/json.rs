//! This module implements specialty functions into [JsonObjectString] to allow it to be serialized
//! and deserialized without adding in escape characters.
use super::super::super::proto_codegen::json::JsonObjectString;
// use super::super::traits::Encode;
use log::error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json;

#[derive(Debug, Clone)]
pub enum JsonStringError {
    InvalidJsonString(String),
}

impl std::fmt::Display for JsonStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidJsonString(string) => {
                write!(f, "Failed to create JsonStringObject: `{}`", string)
            }
            #[allow(unreachable_patterns)]
            _ => std::fmt::Debug::fmt(self, f),
        }
    }
}

impl TryFrom<String> for JsonObjectString {
    type Error = JsonStringError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match serde_json::from_str::<serde_json::Value>(&value) {
            Ok(_) => Ok(Self { jsonstr: value }),
            Err(err) => {
                error!("Not a valid json string `{}` due to: {}", value, err);
                Err(JsonStringError::InvalidJsonString(value))
            }
        }
    }
}

impl TryFrom<&str> for JsonObjectString {
    type Error = JsonStringError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(String::from(value))
    }
}
#[allow(clippy::from_over_into)]
impl Into<String> for JsonObjectString {
    fn into(self) -> String {
        self.jsonstr
    }
}

impl std::fmt::Display for JsonObjectString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.jsonstr)
    }
}

impl Serialize for JsonObjectString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serde_json::from_str::<serde_json::Value>(&self.jsonstr)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }
}

/// Custom implementation of Deserialize, keeps the json value as a string rather than
impl<'de> Deserialize<'de> for JsonObjectString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Deserialize a JSON string into a serde_json::Value object
        let json_val = serde_json::Value::deserialize(deserializer)?;
        // Convert the serde_json::Value object back to a JSON string
        let json_str = json_val.to_string();
        Ok(JsonObjectString { jsonstr: json_str })
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    impl JsonObjectString {
        fn test_serialize(string: &str) {
            assert_eq!(
                JsonObjectString::try_from(string).expect("").jsonstr,
                string,
                "Failed to convert: `{:?}`",
                string
            )
        }
    }
    /// [Example1] tests a simple key value pair
    pub const EXAMPLE1: &str = r#"{"key": "value"}"#;
    /// [Example2] is an example of a numerical array
    pub const EXAMPLE2: &str = r#"{"array": [1, 2, 3]}"#;
    /// [Example3] is an example with nested values
    pub const EXAMPLE3: &str = r#"{"nested": {"foo": "bar"}}"#;
    /// [Example4] is a boolean example
    pub const EXAMPLE4: &str = r#"{"boolean": true}"#;
    /// [Example5] is a null value
    pub const EXAMPLE5: &str = r#"{"null_value": null}"#;
    /// [Example6] is an example of a string array
    pub const EXAMPLE6: &str = r#"{"array": ["one", "two", "three"]}"#;

    #[test]
    fn test_serialize_examples() {
        JsonObjectString::test_serialize(EXAMPLE1);
        JsonObjectString::test_serialize(EXAMPLE2);
        JsonObjectString::test_serialize(EXAMPLE3);
        JsonObjectString::test_serialize(EXAMPLE4);
        JsonObjectString::test_serialize(EXAMPLE5);
        JsonObjectString::test_serialize(EXAMPLE6);
    }
}
