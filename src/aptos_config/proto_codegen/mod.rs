pub mod aptos {
    pub mod modules {
        include!("aptos.modules.rs");
    }
    pub mod records {
        include!("aptos.records.rs");
    }
    pub mod table_items {
        include!("aptos.table_items.rs");
    }
    pub mod resource_extras {
        include!("aptos.resource_extras.rs");
    }
    pub mod common {
        include!("aptos.common.rs");
    }
    pub mod pubsub_range {
        include!("aptos.pubsub_range.rs");
    }
    pub mod transactions {
        include!("aptos.transactions.rs");
    }
    pub mod changes {
        include!("aptos.changes.rs");
    }
    pub mod signatures {
        include!("aptos.signatures.rs");
    }
    pub mod blocks {
        include!("aptos.blocks.rs");
    }
}
pub mod json {
    include!("json.rs");
    pub mod resources {
        include!("json.resources.rs");
    }
    pub mod events {
        include!("json.events.rs");
    }
}
