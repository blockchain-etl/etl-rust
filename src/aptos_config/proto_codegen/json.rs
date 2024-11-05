// This file is @generated by prost-build.
/// The json object string
///
/// When serializing, should be serialized as an json object string, not a json string value
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JsonObjectString {
    /// The Json String stored in this
    #[prost(string, required, tag = "1")]
    pub jsonstr: ::prost::alloc::string::String,
}