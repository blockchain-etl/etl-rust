// Get all the enums that should appear from codegen
use super::proto_codegen::{
    aptos::changes::change::ChangeType,
    aptos::common::Visibility,
    aptos::modules::module::MoveAbility,
    aptos::resource_extras::ResourceChangeType,
    aptos::signatures::signature::{
        public_key::PublicKeyType, signature::SignatureType, SignatureBuildType,
    },
    aptos::table_items::table_item::TableChangeType,
    aptos::transactions::transaction::{PayloadType, TxType},
};

// Macro to implement Into<String> for enums with an `as_str_name` method
#[macro_export]
macro_rules! impl_to_string_for_enum {
    ($t:ty) => {
        #[allow(clippy::from_over_into)]
        impl Into<String> for $t {
            fn into(self) -> String {
                String::from(self.as_str_name().to_string())
            }
        }
    };
}

// Automatically derive Into<String> through the ToString implementation
// by utilizing prost's automatic as_str_name()
impl_to_string_for_enum!(ChangeType);
impl_to_string_for_enum!(Visibility);
impl_to_string_for_enum!(MoveAbility);
impl_to_string_for_enum!(ResourceChangeType);
impl_to_string_for_enum!(SignatureType);
impl_to_string_for_enum!(SignatureBuildType);
impl_to_string_for_enum!(TableChangeType);
impl_to_string_for_enum!(PayloadType);
impl_to_string_for_enum!(TxType);
impl_to_string_for_enum!(PublicKeyType);
