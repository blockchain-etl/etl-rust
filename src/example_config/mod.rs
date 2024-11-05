// TODO: this file will contain the high-level logic (glue).
//  e.g. main() will call the function in this file for the indexing logic as well as the data extraction and record outputting

/// This function creates a pubsub subscription to create requests for what tx versions we
/// want to index.
#[cfg(feature = "ORCHESTRATED")]
pub async fn subscribe_and_extract(
    pubsub_subscription: google_cloud_pubsub::subscription::Subscription,
    publisher: blockchain_generic::output::publish::StreamPublisher,
    metrics: Option<Metrics>,
) -> Result<(), ExtractionInteruptionError> {
    todo!("write subscribe_and_extract function");
}

/// The primary function for indexing, creates a stream for a given range and
/// creates records to be sent to publishers based off the aptos stream response.
pub async fn extract_range(
    start: u64,
    end: u64,
    publisher: blockchain_generic::output::publish::StreamPublisher,
    metrics: Option<Metrics>,
    table_options: Option<TableOptions>,
) -> Result<(), ExtractionInteruptionError> {
    todo!("write extract_range function")
}

/// Extracts transactions, returns a Vec. If
/// provided an `outdir` [PathBuf], should serialize the values and save it in the directory.
pub async fn extract_txs(
    start: u64,
    end: u64,
    outdir: Option<PathBuf>,
) -> Result<Vec<aptos_protos::transaction::v1::Transaction>, ExtractionInteruptionError> {
    todo!("write extract_txs function")
}
