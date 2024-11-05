// This file is @generated by prost-build.
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UnixTimestamp {
    #[prost(int64, required, tag = "1")]
    pub seconds: i64,
    #[prost(uint32, required, tag = "2")]
    pub nanos: u32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Visibility {
    Private = 1,
    Public = 2,
    Friend = 3,
}
impl Visibility {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Visibility::Private => "private",
            Visibility::Public => "public",
            Visibility::Friend => "friend",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "private" => Some(Self::Private),
            "public" => Some(Self::Public),
            "friend" => Some(Self::Friend),
            _ => None,
        }
    }
}