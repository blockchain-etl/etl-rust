// This file is @generated by prost-build.
/// / Represents a singular module output record
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Module {
    #[prost(int64, required, tag = "1")]
    pub block_height: i64,
    #[prost(string, required, tag = "2")]
    pub block_timestamp: ::prost::alloc::string::String,
    #[prost(uint64, required, tag = "3")]
    pub tx_version: u64,
    #[prost(string, required, tag = "4")]
    pub tx_hash: ::prost::alloc::string::String,
    #[prost(uint64, required, tag = "5")]
    pub change_index: u64,
    #[prost(string, optional, tag = "6")]
    pub bytecode: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "7")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "8")]
    pub name: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(message, repeated, tag = "9")]
    pub friends: ::prost::alloc::vec::Vec<module::Friend>,
    #[prost(message, repeated, tag = "10")]
    pub exposed_functions: ::prost::alloc::vec::Vec<module::ExposedFunction>,
    #[prost(message, repeated, tag = "11")]
    pub structs: ::prost::alloc::vec::Vec<module::Struct>,
    #[prost(message, required, tag = "12")]
    pub block_unixtimestamp: super::common::UnixTimestamp,
}
/// Nested message and enum types in `Module`.
pub mod module {
    /// Modules that are similar
    #[derive(serde::Serialize, serde::Deserialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Friend {
        #[prost(string, required, tag = "1")]
        pub address: ::prost::alloc::string::String,
        #[prost(string, required, tag = "2")]
        pub name: ::prost::alloc::string::String,
    }
    /// Exposed functions in the module abi
    #[derive(serde::Serialize, serde::Deserialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExposedFunction {
        /// The name of the exposed function
        #[prost(string, required, tag = "1")]
        pub name: ::prost::alloc::string::String,
        /// the visibility of the function, should be mapped from a similar enum in aptos-protos
        #[prost(string, required, tag = "2")]
        pub visibility: ::prost::alloc::string::String,
        #[prost(bool, required, tag = "3")]
        pub is_entry: bool,
        #[prost(message, repeated, tag = "5")]
        pub generic_type_params: ::prost::alloc::vec::Vec<
            exposed_function::GenericFunctionTypeParams,
        >,
        #[prost(string, repeated, tag = "6")]
        pub params: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        #[prost(string, repeated, tag = "7")]
        pub r#return: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    }
    /// Nested message and enum types in `ExposedFunction`.
    pub mod exposed_function {
        /// Contains the ability constraints applied to the params
        #[derive(serde::Serialize, serde::Deserialize)]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct GenericFunctionTypeParams {
            /// The ability constraints
            #[prost(string, repeated, tag = "1")]
            pub constraints: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        }
    }
    /// Structs in the module abi
    #[derive(serde::Serialize, serde::Deserialize)]
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Struct {
        /// The name of the struct
        #[prost(string, required, tag = "1")]
        pub name: ::prost::alloc::string::String,
        #[prost(bool, required, tag = "2")]
        pub is_native: bool,
        #[prost(string, repeated, tag = "3")]
        pub abilities: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
        #[prost(message, repeated, tag = "4")]
        pub generic_type_params: ::prost::alloc::vec::Vec<
            r#struct::GenericStructTypeParams,
        >,
        #[prost(message, repeated, tag = "5")]
        pub fields: ::prost::alloc::vec::Vec<r#struct::Fields>,
    }
    /// Nested message and enum types in `Struct`.
    pub mod r#struct {
        /// Contains the ability constraints applied to the params
        #[derive(serde::Serialize, serde::Deserialize)]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct GenericStructTypeParams {
            /// The ability constraints
            #[prost(string, repeated, tag = "1")]
            pub constraints: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
            /// Phantom
            #[prost(bool, required, tag = "2")]
            pub is_phantom: bool,
        }
        /// Individual fields in the struct
        #[derive(serde::Serialize, serde::Deserialize)]
        #[allow(clippy::derive_partial_eq_without_eq)]
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct Fields {
            /// Name of the field
            #[prost(string, required, tag = "1")]
            pub name: ::prost::alloc::string::String,
            /// String representing the type of the field
            #[prost(string, required, tag = "2")]
            pub r#type: ::prost::alloc::string::String,
        }
    }
    /// Represents the different abilities that can be applied, should be mapped from a similar enum in aptos-protos
    #[derive(serde::Serialize, serde::Deserialize)]
    #[derive(
        Clone,
        Copy,
        Debug,
        PartialEq,
        Eq,
        Hash,
        PartialOrd,
        Ord,
        ::prost::Enumeration
    )]
    #[repr(i32)]
    pub enum MoveAbility {
        Copy = 1,
        Drop = 2,
        Store = 3,
        Key = 4,
    }
    impl MoveAbility {
        /// String value of the enum field names used in the ProtoBuf definition.
        ///
        /// The values are not transformed in any way and thus are considered stable
        /// (if the ProtoBuf definition does not change) and safe for programmatic use.
        pub fn as_str_name(&self) -> &'static str {
            match self {
                MoveAbility::Copy => "Copy",
                MoveAbility::Drop => "Drop",
                MoveAbility::Store => "Store",
                MoveAbility::Key => "Key",
            }
        }
        /// Creates an enum from field names used in the ProtoBuf definition.
        pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
            match value {
                "Copy" => Some(Self::Copy),
                "Drop" => Some(Self::Drop),
                "Store" => Some(Self::Store),
                "Key" => Some(Self::Key),
                _ => None,
            }
        }
    }
}
