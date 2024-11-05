use super::super::changes::{ChangeError, IncompleteChangeRecord};
use super::super::hashval::HashValue;
use crate::blockchain_config::proto_codegen::aptos::transactions::transaction::ChangesAggregate;
use aptos_protos::transaction::v1 as input_protos;

#[derive(Debug, Clone)]
pub enum TxInfoExtractionError {
    MissingStateCheckpointHash,
    Change(ChangeError),
    UnaccountedForChanges(Vec<input_protos::write_set_change::Type>),
}

impl From<ChangeError> for TxInfoExtractionError {
    fn from(value: ChangeError) -> Self {
        Self::Change(value)
    }
}

#[derive(Debug, Clone)]
pub struct TransactionInfoExtraction {
    pub hash: HashValue,
    pub state_change_hash: HashValue,
    pub event_root_hash: HashValue,
    pub state_checkpoint_hash: Option<HashValue>,
    pub gas_used: u64,
    pub success: bool,
    pub vm_status: String,
    pub accumulator_root_hash: HashValue,
    pub changes: Vec<IncompleteChangeRecord>,
}

impl TransactionInfoExtraction {
    /// Creates the ChangesAggregate, a struct containing change counts per type.
    #[inline]
    pub fn calculate_changes_aggregate(&self) -> Result<ChangesAggregate, TxInfoExtractionError> {
        // Count by using variants as keys
        let mut counts = std::collections::HashMap::with_capacity(8);
        for change in self.changes.iter() {
            *counts.entry(change.get_type()).or_insert(0) += 1;
        }
        // Calculate the Change Aggregate
        let agg = ChangesAggregate {
            total: self.changes.len() as u64,
            delete_module: counts
                .remove(&input_protos::write_set_change::Type::DeleteModule)
                .unwrap_or(0),
            write_module: counts
                .remove(&input_protos::write_set_change::Type::WriteModule)
                .unwrap_or(0),
            delete_resource: counts
                .remove(&input_protos::write_set_change::Type::DeleteResource)
                .unwrap_or(0),
            write_resource: counts
                .remove(&input_protos::write_set_change::Type::WriteResource)
                .unwrap_or(0),
            delete_table_item: counts
                .remove(&input_protos::write_set_change::Type::DeleteTableItem)
                .unwrap_or(0),
            write_table_item: counts
                .remove(&input_protos::write_set_change::Type::WriteTableItem)
                .unwrap_or(0),
        };

        // If counts isn't empty, then there is a change type we did not account for.  If it is empty, we can return the agg,
        // otherwise we return an error stating what is unaccounted for.
        match counts.is_empty() {
            true => Ok(agg),
            false => Err(TxInfoExtractionError::UnaccountedForChanges(
                Vec::from_iter(counts.keys().copied()),
            )),
        }
    }
}

impl TryFrom<input_protos::TransactionInfo> for TransactionInfoExtraction {
    type Error = TxInfoExtractionError;

    fn try_from(value: input_protos::TransactionInfo) -> Result<Self, Self::Error> {
        Ok(Self {
            hash: value.hash.into(),
            state_change_hash: value.state_change_hash.into(),
            event_root_hash: value.event_root_hash.into(),
            state_checkpoint_hash: value.state_checkpoint_hash.map(|bytes| bytes.into()),
            gas_used: value.gas_used,
            success: value.success,
            vm_status: value.vm_status.clone(),
            accumulator_root_hash: value.accumulator_root_hash.into(),
            changes: {
                let mut changes = Vec::new();
                for wsc in value.changes {
                    changes.push(IncompleteChangeRecord::try_from(wsc)?);
                }
                changes
            },
        })
    }
}
