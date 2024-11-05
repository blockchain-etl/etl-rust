// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Resource {
    #[prost(int64, required, tag = "1")]
    pub block_height: i64,
    #[prost(string, required, tag = "2")]
    pub block_timestamp: ::prost::alloc::string::String,
    #[prost(uint64, required, tag = "3")]
    pub tx_version: u64,
    #[prost(string, required, tag = "4")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, optional, tag = "5")]
    pub tx_sequence_number: ::core::option::Option<u64>,
    #[prost(uint64, required, tag = "6")]
    pub change_index: u64,
    #[prost(string, required, tag = "7")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, required, tag = "8")]
    pub state_key_hash: ::prost::alloc::string::String,
    #[prost(string, required, tag = "9")]
    pub change_type: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "10")]
    pub struct_tag: ::core::option::Option<
        super::super::aptos::resource_extras::StructTag,
    >,
    #[prost(string, required, tag = "11")]
    pub type_str: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "12")]
    pub resource: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, required, tag = "13")]
    pub block_unixtimestamp: super::super::aptos::common::UnixTimestamp,
}