use crate::aptos_config::proto_codegen::aptos::pubsub_range::{IndexingRange, TableOptions};

impl IndexingRange {
    #[inline]
    pub fn start(&self) -> u64 {
        self.start
    }

    #[inline]
    pub fn end(&self) -> u64 {
        self.end
    }

    #[inline]
    pub fn table_options(&self) -> TableOptions {
        match &self.table {
            Some(options) => options.clone(),
            None => TableOptions::new_all(),
        }
    }
}

impl TableOptions {
    #[inline]
    pub const fn new_all() -> Self {
        Self {
            blocks: Some(true),
            transactions: Some(true),
            events: Some(true),
            changes: Some(true),
            modules: Some(true),
            table_items: Some(true),
            resource: Some(true),
            signatures: Some(true),
        }
    }

    #[inline]
    pub const fn new_all_false() -> Self {
        Self {
            blocks: Some(false),
            transactions: Some(false),
            events: Some(false),
            changes: Some(false),
            modules: Some(false),
            table_items: Some(false),
            resource: Some(false),
            signatures: Some(false),
        }
    }

    #[inline]
    pub const fn new_all_none() -> Self {
        Self {
            blocks: None,
            transactions: None,
            events: None,
            changes: None,
            modules: None,
            table_items: None,
            resource: None,
            signatures: None,
        }
    }

    #[inline]
    pub fn do_blocks(&self) -> bool {
        self.blocks.unwrap_or(false)
    }

    #[inline]
    pub fn do_signatures(&self) -> bool {
        self.signatures.unwrap_or(false)
    }

    #[inline]
    pub fn do_transactions(&self) -> bool {
        self.transactions.unwrap_or(false)
    }

    #[inline]
    pub fn do_events(&self) -> bool {
        self.events.unwrap_or(false)
    }

    #[inline]
    pub fn do_changes(&self) -> bool {
        self.changes.unwrap_or(false)
    }

    #[inline]
    pub fn do_modules(&self) -> bool {
        self.modules.unwrap_or(false)
    }

    #[inline]
    pub fn do_resources(&self) -> bool {
        self.resource.unwrap_or(false)
    }

    #[inline]
    pub fn do_table_items(&self) -> bool {
        self.table_items.unwrap_or(false)
    }

    #[inline]
    pub fn do_changes_or_subchanges(&self) -> bool {
        self.do_changes() | self.do_modules() | self.do_resources() | self.do_table_items()
    }
}
