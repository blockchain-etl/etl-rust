use super::blocks::get_block;
use super::changes::get_changes;
use super::events::get_events;
use super::signatures::get_signatures;
use super::transactions::get_transactions;
use crate::{
    aptos_config::proto_codegen::{
        aptos::{
            blocks::Block, changes::Change, modules::Module, signatures::Signature,
            table_items::TableItem, transactions::Transaction,
        },
        json::{events::Event, resources::Resource},
    },
    blockchain_config::{
        processing::reps::transaction::tx::TransactionExtraction,
        proto_codegen::aptos::common::UnixTimestamp,
    },
};
use aptos_protos::transaction::v1::Transaction as AptosTx;
use log::debug;

pub trait StoresRecords<T> {
    /// Returns a vector of the records
    fn vec(&self) -> &Vec<T>;
    /// Returns a mutable vector of records
    fn mutvec(&mut self) -> &mut Vec<T>;

    /// Allows adding an item to the records
    #[inline]
    fn add(&mut self, record: T) {
        self.mutvec().push(record);
    }
    /// Allows adding multiple records from a vector via [Vec::extend]
    #[inline]
    fn add_vec(&mut self, records: Vec<T>) {
        self.mutvec().extend(records);
    }
    /// Returns if the respective record vec is empty
    #[inline]
    fn is_empty(&self) -> bool {
        self.vec().is_empty()
    }
    /// Returns the length of the respective record vector
    #[inline]
    fn len(&self) -> usize {
        self.vec().len()
    }
}

pub use crate::aptos_config::proto_codegen::aptos::records::Records;

impl Records {
    /// Creates a new [Records] object
    pub fn new() -> Self {
        Self {
            transactions: Vec::new(),
            blocks: Vec::new(),
            table_items: Vec::new(),
            resources: Vec::new(),
            modules: Vec::new(),
            changes: Vec::new(),
            events: Vec::new(),
            signatures: Vec::new(),
        }
    }
    /// Creates a new [Records] object with capacity set to a certain level
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            transactions: Vec::with_capacity(capacity),
            blocks: Vec::with_capacity(capacity),
            table_items: Vec::with_capacity(capacity),
            resources: Vec::with_capacity(capacity),
            modules: Vec::with_capacity(capacity),
            changes: Vec::with_capacity(capacity),
            events: Vec::with_capacity(capacity),
            signatures: Vec::with_capacity(capacity),
        }
    }
    /// Returns the range by getting the min and max tx version by reviewing all the tx records.
    pub fn get_range(&self) -> Option<(u64, u64)> {
        if !self.transactions.is_empty() {
            let tx_versions = self
                .transactions
                .iter()
                .map(|rec| rec.tx_version)
                .collect::<Vec<u64>>();

            match (tx_versions.iter().min(), tx_versions.iter().max()) {
                (Some(start), Some(end)) => Some((*start, *end)),
                (None, None) => None,
                (_, _) => panic!("Somehow had min or max, but not both"),
            }
        } else {
            None
        }
    }

    /// Returns the earliest timestamp after iterating through the transaction records.
    pub fn earliest_timestamp(&self) -> Option<UnixTimestamp> {
        if !self.transactions.is_empty() {
            let mut earliest_timestamp: Option<UnixTimestamp> = None;
            for tx in self.transactions.iter() {
                match &earliest_timestamp {
                    Some(cur_earliest) => {
                        if cur_earliest.seconds > tx.block_unixtimestamp.seconds {
                            earliest_timestamp = Some(tx.block_unixtimestamp.clone())
                        }
                    }
                    None => earliest_timestamp = Some(tx.block_unixtimestamp.clone()),
                }
            }
            earliest_timestamp
        } else {
            None
        }
    }
}

impl StoresRecords<Transaction> for Records {
    fn mutvec(&mut self) -> &mut Vec<Transaction> {
        &mut self.transactions
    }
    fn vec(&self) -> &Vec<Transaction> {
        &self.transactions
    }
}
impl StoresRecords<Block> for Records {
    fn mutvec(&mut self) -> &mut Vec<Block> {
        &mut self.blocks
    }
    fn vec(&self) -> &Vec<Block> {
        &self.blocks
    }
}
impl StoresRecords<TableItem> for Records {
    fn mutvec(&mut self) -> &mut Vec<TableItem> {
        &mut self.table_items
    }
    fn vec(&self) -> &Vec<TableItem> {
        &self.table_items
    }
}
impl StoresRecords<Resource> for Records {
    fn mutvec(&mut self) -> &mut Vec<Resource> {
        &mut self.resources
    }
    fn vec(&self) -> &Vec<Resource> {
        &self.resources
    }
}
impl StoresRecords<Module> for Records {
    fn mutvec(&mut self) -> &mut Vec<Module> {
        &mut self.modules
    }
    fn vec(&self) -> &Vec<Module> {
        &self.modules
    }
}
impl StoresRecords<Change> for Records {
    fn mutvec(&mut self) -> &mut Vec<Change> {
        &mut self.changes
    }
    fn vec(&self) -> &Vec<Change> {
        &self.changes
    }
}
impl StoresRecords<Event> for Records {
    fn mutvec(&mut self) -> &mut Vec<Event> {
        &mut self.events
    }
    fn vec(&self) -> &Vec<Event> {
        &self.events
    }
}
impl StoresRecords<Signature> for Records {
    fn mutvec(&mut self) -> &mut Vec<Signature> {
        &mut self.signatures
    }
    fn vec(&self) -> &Vec<Signature> {
        &self.signatures
    }
}
/// Extracts the records from a transaction
pub fn extract_records(tx: &AptosTx) -> Result<Records, Box<dyn std::error::Error>> {
    let mut records = Records::new();

    debug!("Extracting tx version {}", tx.version);
    let tx_extraction = match TransactionExtraction::try_from(tx.clone()) {
        Ok(tx_extraction) => tx_extraction,
        Err(error) => {
            log::error!("Error extracting transaction");
            return Err(Box::new(error));
        }
    };

    debug!("Extracting block record");
    match get_block(&tx_extraction) {
        Ok(Some(block)) => records.add(block),
        Ok(None) => {}
        Err(err) => {
            log::error!("Error extracting block: {}", err);
            return Err(Box::new(err));
        }
    }

    debug!("Extracting change records (includes: resources, tableitems, modules)");
    match get_changes(&tx_extraction) {
        Ok(changes) => {
            records.add_vec(changes.changes);
            records.add_vec(changes.resources);
            records.add_vec(changes.tableitems);
            records.add_vec(changes.modules);
        }
        Err(err) => {
            log::error!("Error extracting changes: {}", err);
            return Err(Box::new(err));
        }
    }

    debug!("Extracting event records");
    match get_events(&tx_extraction) {
        Ok(Some(events)) => records.add_vec(events),
        Ok(None) => {}
        Err(err) => {
            log::error!("Error extracting events: {}", err);
            return Err(Box::new(err));
        }
    }

    let mut sig_cnt: Option<u64> = None;
    debug!("Extracting signature records");
    match get_signatures(&tx_extraction) {
        Ok(Some(sigs)) => {
            sig_cnt = Some(sigs.len() as u64);
            records.add_vec(sigs);
        }
        Ok(None) => (),
        Err(err) => {
            log::error!("Error extracting signatures: {}", err);
            return Err(Box::new(err));
        }
    };

    debug!("Extracting transaction record");
    match get_transactions(&tx_extraction, sig_cnt) {
        Ok(tx) => records.add(tx),
        Err(err) => {
            log::error!("Error extracting transaction: {}", err);
            return Err(Box::new(err));
        }
    }

    debug!("Extracted tx version {}", tx.version);
    Ok(records)
}
