pub mod enumtostring;
pub mod grpc;
pub mod processing;
#[allow(clippy::module_inception)]
pub mod proto_codegen;
use processing::reps::transaction::tx::{TransactionExtraction, TxExtractionError};
use proto_codegen::aptos::pubsub_range::TableOptions;
use std::io::Write;
use std::{io::Read, path::PathBuf};

#[cfg(feature = "ORCHESTRATED")]
use proto_codegen::aptos::pubsub_range::IndexingRange;
#[cfg(feature = "ORCHESTRATED")]
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
#[cfg(feature = "ORCHESTRATED")]
use tokio::signal::unix::{signal, SignalKind};

#[cfg(test)]
pub mod tests;

#[cfg(feature = "SEPARATE_PUBLISHERS")]
pub mod streampublisher;

use crate::{
    self as blockchain_generic,
    blockchain_config::{
        grpc::stream::{pull_from_stream, StreamPullError},
        // proto_codegen::aptos::changes::change,
    },
};
#[cfg(feature = "METRICS")]
use blockchain_generic::metrics::Metrics;
// use core::ops::Range;
use log::{debug, error, info, warn};
use prost::Message as prost_message;
use std::error::Error;
use std::fs::File;

// use futures_util::StreamExt;

use grpc::{get_stream, StreamCreationError};

use self::{
    processing::tables::{
        blocks::BlockError, changes::ChangeError, events::EventError, signatures::SigProcessError,
        transactions::TxError,
    },
    proto_codegen::aptos::common::UnixTimestamp,
};

/// Number of blocks to index before logging the milestone.
pub const RANGE_SIZE: u64 = 1000;

/// Publishes the records
#[allow(unused_variables)]
async fn publish_records<T>(
    publisher: &blockchain_generic::output::publish::StreamPublisherConnection,
    records: Vec<T>,
    name: Option<&str>,
    timestamp: Vec<UnixTimestamp>,
) where
    T: prost::Message,
    T: serde::Serialize,
{
    debug!("Publishing: {:?}", name);
    #[cfg(feature = "PUBLISH_WITH_NAME")]
    if let Some(output_name) = name {
        #[cfg(feature = "JSON")]
        for (i, record) in records.into_iter().enumerate() {
            publisher
                .publish(&format!("{}_{}", output_name, i), record)
                .await;
        }
        #[cfg(feature = "JSONL")]
        publisher.publish_batch(output_name, records).await;
        #[cfg(feature = "GOOGLE_CLOUD_STORAGE")]
        publisher
            .publish_batch(output_name, timestamp, records)
            .await;
    }

    #[cfg(not(feature = "PUBLISH_WITH_NAME"))]
    {
        #[cfg(feature = "GOOGLE_PUBSUB")]
        publisher.publish_batch(records).await;

        #[cfg(not(feature = "GOOGLE_PUBSUB"))]
        for record in records {
            publisher.publish(record).await;
        }
    }
}

/// Error returned upon receiving an error while extracting.
#[derive(Debug)]
pub enum ExtractError {
    /// Occurs when we fail to create a stream
    StreamCreationError(StreamCreationError),
    /// Occurs when the server sends back a bad status
    StreamErrorStatus(tonic::Status),
    /// Any other Error can optionally be returned here.
    Other(Box<dyn Error>),
    /// When failed to extract information from Transaction. Returns tx number failed on.
    TxExtraction(TxExtractionError),
    /// When failed to create transaction record
    TxRecord(TxError),
    /// When failed to create a signature record(s)
    SignatureRecord(SigProcessError),
    /// When failed to create an event record(s)
    EventRecord(EventError),
    /// When failed to create a change record or one of its subrecords
    ChangeRecord(ChangeError),
    /// When failed to create a Block records
    BlockRecord(BlockError),
    /// When failing to create a file
    FailedToCreateFile(PathBuf, std::io::Error),
    /// When failing to open a file
    FailedToOpenFile(PathBuf, std::io::Error),
    /// When failing to decode proto
    ProtoDecodeError(prost::DecodeError),
}

impl From<StreamCreationError> for ExtractError {
    fn from(value: StreamCreationError) -> Self {
        ExtractError::StreamCreationError(value)
    }
}

impl From<tonic::Status> for ExtractError {
    fn from(value: tonic::Status) -> Self {
        ExtractError::StreamErrorStatus(value)
    }
}

impl From<TxExtractionError> for ExtractError {
    fn from(value: TxExtractionError) -> Self {
        ExtractError::TxExtraction(value)
    }
}

impl From<TxError> for ExtractError {
    fn from(value: TxError) -> Self {
        ExtractError::TxRecord(value)
    }
}

impl From<BlockError> for ExtractError {
    fn from(value: BlockError) -> Self {
        ExtractError::BlockRecord(value)
    }
}

impl From<Box<dyn Error>> for ExtractError {
    fn from(value: Box<dyn Error>) -> Self {
        ExtractError::Other(value)
    }
}

impl From<ChangeError> for ExtractError {
    fn from(value: ChangeError) -> Self {
        Self::ChangeRecord(value)
    }
}

impl From<EventError> for ExtractError {
    fn from(value: EventError) -> Self {
        Self::EventRecord(value)
    }
}

impl From<SigProcessError> for ExtractError {
    fn from(value: SigProcessError) -> Self {
        Self::SignatureRecord(value)
    }
}

impl std::fmt::Display for ExtractError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ExtractError {}

/// An error that wraps ExtractError,
#[derive(Debug)]
pub struct ExtractionInteruptionError {
    pub extracterror: ExtractError,
    pub start_version: u64,
    pub end_version: u64,
    /// The version we failed on.  This means that, unless we failed during publishing,
    /// we have not made any publications on this transaction, nor any moving forward
    pub failed_on: u64,
}

impl ExtractionInteruptionError {
    pub fn new<T>(error: T, start: u64, end: u64, failed_on: u64) -> Self
    where
        T: Into<ExtractError>,
    {
        ExtractionInteruptionError {
            extracterror: error.into(),
            start_version: start,
            end_version: end,
            failed_on,
        }
    }
}

/// Extracts transactions, returns a [Vec]<[aptos_protos::transaction::v1::Transaction]>.  If
/// provided an `outdir` [PathBuf], should serialize the values and save it in the directory.
pub async fn extract_txs(
    start: u64,
    end: u64,
    outdir: Option<PathBuf>,
) -> Result<Vec<aptos_protos::transaction::v1::Transaction>, ExtractionInteruptionError> {
    info!("Attempting to extract transactions [{}, {}]", start, end);

    // Saves the current version
    let mut cur_version: u64 = start;

    // Create the output directory if applicable
    if let Some(outdir) = &outdir {
        match std::fs::create_dir_all(outdir.clone()) {
            Ok(_) => info!("Created directory: {:?}", outdir),
            Err(err) => {
                panic!("Failed to create directory `{:?}`: {}", outdir, err);
            }
        }
    }

    // Get the stream or raise an error
    let mut stream = match get_stream(start, end).await {
        Ok(stream) => stream,
        Err(err) => {
            return Err(ExtractionInteruptionError::new(
                err,
                start,
                end,
                cur_version,
            ));
        }
    };

    // Create transactions vec
    let mut txs: Vec<aptos_protos::transaction::v1::Transaction> = Vec::new();

    // Create a stream and extract the txs
    loop {
        debug!("Checking stream");
        let tx_response = match stream.message().await {
            Ok(Some(txresponse)) => {
                debug!("Received {} transactions", txresponse.transactions.len());
                txresponse
            }
            Ok(None) => {
                info!(
                    "Terminating as we exhausted all transactions from {} to {}",
                    start, end
                );
                return Ok(txs);
            }
            Err(status) => {
                error!("Received Error status: {}", status);
                return Err(ExtractionInteruptionError::new(
                    status,
                    start,
                    end,
                    cur_version,
                ));
            }
        };

        // Iterate through the responded transactions
        for tx in tx_response.transactions.iter() {
            // Update current version
            cur_version = tx.version;

            // Serialize to an output file if provided
            if let Some(outdir) = outdir.clone() {
                // Combine the directory and the file
                let filepath = outdir.join(format!("{}.pb", tx.version));
                // Attempt to create a file
                let mut file = match File::create(filepath.clone()) {
                    Ok(file) => file,
                    Err(err) => {
                        error!(
                            "Failed to create file while creating `{:?}` due to error: {}",
                            filepath, err
                        );
                        return Err(ExtractionInteruptionError::new(
                            ExtractError::FailedToCreateFile(filepath, err),
                            start,
                            end,
                            tx.version,
                        ));
                    }
                };

                // Serialize the tx
                let serialized_tx = tx.encode_to_vec();

                // Write to file,
                match file.write_all(&serialized_tx) {
                    Ok(_) => debug!("Serialized tx {}", tx.version),
                    Err(err) => {
                        error!("Failed to write to file `{:?}`: {}", filepath, err);
                        return Err(ExtractionInteruptionError::new(
                            ExtractError::Other(Box::new(err)),
                            start,
                            end,
                            cur_version,
                        ));
                    }
                }
            }
            txs.push(tx.clone());
        }
    }
}

/// Transforms transactions
pub async fn transform_txs(
    txs: Vec<aptos_protos::transaction::v1::Transaction>,
    outdir: Option<&PathBuf>,
    publisher: Option<blockchain_generic::output::publish::StreamPublisher>,
) -> Result<Vec<proto_codegen::aptos::records::Records>, ExtractError> {
    // If given an output directory, create it
    if let Some(outdir) = &outdir {
        // Create the directory
        match std::fs::create_dir_all(outdir) {
            Ok(_) => info!("Created {:?}", outdir),
            Err(err) => {
                error!("Failed to create directory: {}", err);
                return Err(ExtractError::Other(Box::new(err)));
            }
        }
    }

    let mut records_vec = Vec::new();

    // Go through the transactions
    for tx in txs.iter() {
        // Get the extracted records
        let records = match extract_records(tx) {
            Ok(records) => records,
            Err(err) => {
                error!("Error occured whenever trying to extract records: {}", err);
                return Err(ExtractError::Other(err));
            }
        };

        records_vec.push(records.clone());

        if let Some(outdir) = outdir {
            let curpath = outdir.join(format!("{}.pb", tx.version));
            save_records(records.clone(), &curpath).await?
        };

        // If given a publisher, publish it
        if let Some(publisher) = &publisher {
            // Attempt to get the range
            let name_str = match records.get_range() {
                Some((start, end)) => {
                    format!("{}_{}", start, end)
                }
                None => String::from(""),
            };
            // Get as a Option<&str>
            let name = match name_str.is_empty() {
                true => Some(name_str.as_str()),
                false => None,
            };
            #[allow(unused_variables)]
            let timestamp = records.earliest_timestamp();

            // Send to publish
            publish_record_struct(publisher, records, name).await;
        }
    }

    Ok(records_vec)
}

/// Sends each piece of the Record Struct into the publisher.
pub async fn publish_record_struct(
    publisher: &blockchain_generic::output::publish::StreamPublisher,
    records: proto_codegen::aptos::records::Records,
    name: Option<&str>,
) {
    if !records.blocks.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .blocks
            .iter()
            .map(|block| block.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.blocks, records.blocks, name, timestamps).await
    }
    if !records.transactions.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .transactions
            .iter()
            .map(|tx| tx.block_unixtimestamp.clone())
            .collect();
        publish_records(
            &publisher.transactions,
            records.transactions,
            name,
            timestamps,
        )
        .await
    }
    if !records.signatures.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .signatures
            .iter()
            .map(|sig| sig.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.signatures, records.signatures, name, timestamps).await
    }
    if !records.events.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .events
            .iter()
            .map(|event| event.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.events, records.events, name, timestamps).await
    }
    if !records.changes.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .changes
            .iter()
            .map(|ch| ch.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.changes, records.changes, name, timestamps).await
    }
    if !records.resources.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .resources
            .iter()
            .map(|res| res.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.resources, records.resources, name, timestamps).await
    }
    if !records.table_items.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .table_items
            .iter()
            .map(|res| res.block_unixtimestamp.clone())
            .collect();
        publish_records(
            &publisher.table_items,
            records.table_items,
            name,
            timestamps,
        )
        .await
    }
    if !records.modules.is_empty() {
        let timestamps: Vec<UnixTimestamp> = records
            .modules
            .iter()
            .map(|module| module.block_unixtimestamp.clone())
            .collect();
        publish_records(&publisher.modules, records.modules, name, timestamps).await
    }
}

/// This function creates a pubsub subscription to create requests for what tx versions we
/// want to index.
#[cfg(feature = "ORCHESTRATED")]
pub async fn subscribe_and_extract(
    pubsub_subscription: google_cloud_pubsub::subscription::Subscription,
    publisher: blockchain_generic::output::publish::StreamPublisher,
    metrics: Option<Metrics>,
) -> Result<(), ExtractionInteruptionError> {
    info!("Starting the indexer...");

    // shared between the terminator thread for signal handling, and the main subscriber thread.
    let terminated = Arc::new(AtomicBool::new(false));

    // spawns a thread that just listens for a SIGTERM signal, and sets a shutdown flag upon receiving it.
    let terminator = terminated.clone();
    tokio::spawn(async move {
        let mut sigterm =
            signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");
        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to set up SIGINT handler");

        // hangs until receiving the signal.
        //sigterm.recv().await;
        tokio::select! {
            _ = sigterm.recv() => {
                warn!("SIGTERM received, shutting down gracefully...");
            },
            _ = sigint.recv() => {
                warn!("SIGINT received, shutting down gracefully...");
            },
        }

        // set terminated flag to true to indicate a shutdown should occur.
        // uses release ordering because this is the writer thread.
        terminator.store(true, Ordering::Release);
    });

    // continually pulls pub/sub messages to determine which ranges to index.
    // uses acquire ordering for reader thread
    // TODO: add a timer to this .await and continue to the next iteration. this ensures that if there is no message in the pub/sub subscription, then this instance can still be shutdown (otherwise this .await will hang)

    while !terminated.load(Ordering::Acquire) {
        let message = match pubsub_subscription.pull(1, None).await {
            Ok(mut o) => match o.pop() {
                None => {
                    warn!("Didn't receive a message from the subscription. Retrying...");
                    continue;
                }
                Some(m) => m,
            },
            Err(e) => {
                warn!(
                    "Could not pull a pub/sub message from the subscription: {:?}. Retrying...",
                    e
                );
                continue;
            }
        };

        info!("Got Message: {:?}", message.message);

        // deserialize the pub/sub message into an indexing range
        let cur_range: IndexingRange = {
            let message_ref: &[u8] = message.message.data.as_ref();
            IndexingRange::decode(message_ref)
                .expect("pub/sub message uses the IndexingRange protobuf format")
        };

        // NOTE: extract_range() includes the bounds of the range, like [start, end]
        let start = cur_range.start;
        let end = cur_range.end;
        let tables = cur_range.table;

        match extract_range(start, end, publisher.clone(), metrics.clone(), tables).await {
            Err(extract_error) => {
                match message.nack().await {
                    Ok(_) => error!("Nacked the message due to extraction or transformation error"),
                    Err(status) => {
                        error!("Nack returned a status: {:?}", status);
                    }
                };
                return Err(extract_error);
            }
            Ok(_) => info!("Extraction and transformation succeeded"),
        }

        // ack the message to prevent the message from being re-delivered.
        match message.ack().await {
            Ok(_) => info!("Acked the message"),
            Err(status) => {
                info!("Ack returned a status: {:?}", status);
                continue;
            }
        };
    }

    info!("Received a shutdown signal. Shutting down...");

    Ok(())
}

/// This function takes an iterable and sorts as necessary to pass to extract_range.
#[cfg(not(feature = "ORCHESTRATED"))]
pub async fn extract<I: Iterator<Item = u64>>(
    indexing_iter: I,
    publisher: blockchain_generic::output::publish::StreamPublisher,
    metrics: Option<Metrics>,
    tables: Option<TableOptions>,
) -> Result<(), ExtractionInteruptionError> {
    info!("Starting the indexer for iterator");

    // Turns the iterator into a sorted vector with only unique values
    let version_list: Vec<u64> = {
        let mut vector: Vec<u64> = indexing_iter.collect();
        vector.sort_unstable();
        vector.dedup();
        vector
    };

    // If no values, go ahead and return Ok
    if version_list.is_empty() {
        warn!("Received an empty list. Doing nothing...");
        return Ok(());
    }

    // Get the first and last versions from our list.
    let first_version = match version_list.first() {
        Some(first) => *first,
        None => unreachable!("first_version can only be None if version_list is empty"),
    };
    let last_version = match version_list.first() {
        Some(last) => *last,
        None => unreachable!("first_version can only be None if version_list is empty"),
    };

    // Call extract_range
    extract_range(first_version, last_version, publisher, metrics, tables).await
}

/// The primary extraction function for Aptos indexing, creates a stream for a given range and
/// creates records to be sent to publishers based off the aptos stream response.
pub async fn extract_range(
    start: u64,
    end: u64,
    publisher: blockchain_generic::output::publish::StreamPublisher,
    metrics: Option<Metrics>,
    table_options: Option<TableOptions>,
) -> Result<(), ExtractionInteruptionError> {
    // prepare the publisher, if necessary:
    #[cfg(feature = "RABBITMQ_CLASSIC")]
    let publisher = publisher.with_channel().await;
    #[cfg(feature = "APACHE_KAFKA")]
    let publisher = publisher.with_producer().await;

    let table_options = match table_options {
        Some(tbl_opts) => tbl_opts,
        // None => TableOptions::new_all(),
        None => TableOptions::new_all(),
    };

    info!("Starting the indexer for range {} to {}...", start, end);

    let mut cur_version: u64 = start;

    if let Some(m) = &metrics {
        m.request_count.inc();
    };

    // Get the stream (or raise an err)
    //  This is the part where we actually request the data
    let mut stream = match get_stream(start, end).await {
        Ok(stream) => stream,
        Err(err) => {
            if let Some(m) = &metrics {
                m.failed_request_count.inc();
            };
            return Err(ExtractionInteruptionError::new(
                err,
                start,
                end,
                cur_version,
            ));
        }
    };

    let num_tx = (end - start) as usize;
    let mut all_tx_records = Vec::with_capacity(num_tx);
    let mut all_block_records = Vec::new();
    let mut all_change_records = Vec::with_capacity(num_tx);
    let mut all_signature_records = Vec::new();
    let mut all_event_records = Vec::with_capacity(num_tx);
    let mut all_table_item_records = Vec::new();
    let mut all_module_records = Vec::new();
    let mut all_resource_records = Vec::with_capacity(num_tx);

    let mut all_tx_timestamps = Vec::with_capacity(num_tx);
    let mut all_block_timestamps = Vec::new();
    let mut all_change_timestamps = Vec::with_capacity(num_tx);
    let mut all_signature_timestamps = Vec::new();
    let mut all_event_timestamps = Vec::with_capacity(num_tx);
    let mut all_table_item_timestamps = Vec::new();
    let mut all_module_timestamps = Vec::new();
    let mut all_resource_timestamps = Vec::with_capacity(num_tx);

    // Keep pulling from queueing from the stream until stream terminates
    loop {
        // This is where we pull the data from the stream.
        debug!("Checking Stream");
        let tx_response = match pull_from_stream(&mut stream).await {
            Ok(txresponse) => txresponse,
            Err(StreamPullError::Status(status)) => {
                return Err(ExtractionInteruptionError::new(
                    status,
                    start,
                    end,
                    cur_version,
                ))
            }
            Err(StreamPullError::Exhausted) => {
                info!(
                    "Terminating, exhausted all transactions from {} to {}",
                    start, end
                );
                return Ok(());
            }
        };

        // Here is where we would do the conversions to our records,
        //  NOTE: I put just Tx Version # for now so you can get a gist if your pubsub requests are working!
        for tx in tx_response.transactions {
            cur_version = tx.version;

            debug!("[Tx {}] Starting", cur_version);

            // Start with creating an extraction of the transaction
            let tx_extract = match TransactionExtraction::try_from(tx) {
                Ok(extract) => extract,
                Err(error) => {
                    error!(
                        "Received error while extracting from transaction: {:?}",
                        error
                    );
                    return Err(ExtractionInteruptionError::new(
                        error,
                        start,
                        end,
                        cur_version,
                    ));
                }
            };

            // Parse and create all the records.  To reduce duplicates, we do not want to start
            // publishing until we ensure that no errors occur.  (or if we switch to being able
            // to specify tables, then we can just track which tables were successful and which weren't)
            #[allow(unused_assignments)]
            let mut sig_cnt: Option<u64> = None;

            // Create the records for signatures.  If not doing signatures, some work still needs
            // to be done in order to get signature counts.

            if table_options.do_signatures() || table_options.do_transactions() {
                debug!("[Tx {}] Signature Records", cur_version);
                match processing::tables::signatures::get_signatures(&tx_extract) {
                    Ok(signatures) => {
                        sig_cnt = signatures.as_ref().map(|sigs| sigs.len() as u64);
                        // If we are doing signatures, we need to do more work other than counting
                        if table_options.do_signatures() {
                            #[allow(clippy::map_flatten)]
                            let signature_timestamps: Vec<
                                UnixTimestamp,
                            > = signatures
                                .iter()
                                .map(|sig| sig.iter().map(|sig| sig.block_unixtimestamp.clone()))
                                .flatten()
                                .collect();
                            all_signature_timestamps.extend(signature_timestamps);
                            signatures
                                .into_iter()
                                .for_each(|recs| all_signature_records.extend(recs));
                        }
                    }
                    Err(error) => {
                        error!(
                            "Received error while handling signature record(s): {:?}",
                            error
                        );
                        return Err(ExtractionInteruptionError::new(
                            error,
                            start,
                            end,
                            cur_version,
                        ));
                    }
                };
            }

            //  Create block record if applicable
            if table_options.do_blocks() {
                debug!("[Tx {}] Block Records", cur_version);
                match processing::tables::blocks::get_block(&tx_extract) {
                    Ok(block_rec) => {
                        if let Some(contents) = block_rec {
                            all_block_timestamps.push(contents.block_unixtimestamp.clone());
                            all_block_records.push(contents);
                        }
                    }
                    Err(error) => {
                        error!(
                            "Received error while handling the block record: {:?}",
                            error
                        );
                        return Err(ExtractionInteruptionError::new(
                            error,
                            start,
                            end,
                            cur_version,
                        ));
                    }
                };
            }

            //  Create transaction record
            if table_options.do_transactions() {
                debug!("[Tx {}] Transaction Records", cur_version);
                match processing::tables::transactions::get_transactions(&tx_extract, sig_cnt) {
                    Ok(tx_rec) => {
                        all_tx_timestamps.push(tx_rec.block_unixtimestamp.clone());
                        all_tx_records.push(tx_rec);
                    }
                    Err(error) => {
                        error!(
                            "Received error while handling the transaction record: {:?}",
                            error
                        );
                        return Err(ExtractionInteruptionError::new(
                            error,
                            start,
                            end,
                            cur_version,
                        ));
                    }
                };
            }

            if table_options.do_events() {
                debug!("[Tx {}] Event Records", cur_version);
                match processing::tables::events::get_events(&tx_extract) {
                    Ok(events) => {
                        #[allow(clippy::map_flatten)]
                        let events_timestamps: Vec<UnixTimestamp> = events
                            .iter()
                            .map(|event| event.iter().map(|ev| ev.block_unixtimestamp.clone()))
                            .flatten()
                            .collect();
                        all_event_timestamps.extend(events_timestamps);
                        events
                            .into_iter()
                            .for_each(|recs| all_event_records.extend(recs));
                    }
                    Err(error) => {
                        error!("Received error while handling event record(s): {:?}", error);
                        return Err(ExtractionInteruptionError::new(
                            error,
                            start,
                            end,
                            cur_version,
                        ));
                    }
                };
            }

            if table_options.do_changes_or_subchanges() {
                debug!("[Tx {}] Change Records & SubRecords", cur_version);
                match processing::tables::changes::get_changes(&tx_extract) {
                    Ok(change_records) => {
                        if table_options.do_changes() {
                            let change_timestamps: Vec<UnixTimestamp> = change_records
                                .changes
                                .iter()
                                .map(|ch| ch.block_unixtimestamp.clone())
                                .collect();
                            all_change_timestamps.extend(change_timestamps);
                            all_change_records.extend(change_records.changes);
                        }
                        if table_options.do_resources() {
                            let resource_timestamps: Vec<UnixTimestamp> = change_records
                                .resources
                                .iter()
                                .map(|re| re.block_unixtimestamp.clone())
                                .collect();
                            all_resource_timestamps.extend(resource_timestamps);
                            all_resource_records.extend(change_records.resources);
                        }
                        if table_options.do_modules() {
                            let module_timestamps: Vec<UnixTimestamp> = change_records
                                .modules
                                .iter()
                                .map(|mo| mo.block_unixtimestamp.clone())
                                .collect();
                            all_module_timestamps.extend(module_timestamps);
                            all_module_records.extend(change_records.modules);
                        }
                        if table_options.do_table_items() {
                            let table_item_timestamps: Vec<UnixTimestamp> = change_records
                                .tableitems
                                .iter()
                                .map(|t| t.block_unixtimestamp.clone())
                                .collect();
                            all_table_item_timestamps.extend(table_item_timestamps);
                            all_table_item_records.extend(change_records.tableitems);
                        }
                    }
                    Err(error) => {
                        error!("Received error while handling change record(s) and change subrecords: {:?}", error);
                        return Err(ExtractionInteruptionError::new(
                            error,
                            start,
                            end,
                            cur_version,
                        ));
                    }
                };
            }

            debug!("Tx Version Completed: {}", cur_version);
            if cur_version == end {
                //info!("Reached tx #{}", cur_version);
                break;
            }
        }

        if cur_version == end {
            info!("Reached tx #{}", cur_version);
            break;
        }
    }

    if table_options.do_transactions() {
        debug!("[Tx {}] Publishing transaction record", cur_version);
        publish_records(
            &publisher.transactions,
            all_tx_records,
            Some(&format!("{}_{}", start, end)),
            all_tx_timestamps,
        )
        .await;
    }

    if table_options.do_changes() {
        debug!(
            "[Tx {}] Publishing {} change records",
            cur_version,
            all_change_records.len()
        );
        publish_records(
            &publisher.changes,
            all_change_records,
            Some(&format!("{}_{}", start, end)),
            all_change_timestamps,
        )
        .await;
    }

    if table_options.do_modules() {
        debug!(
            "[Tx {}] Publishing {} module records",
            cur_version,
            all_module_records.len()
        );
        publish_records(
            &publisher.modules,
            all_module_records,
            Some(&format!("{}_{}", start, end)),
            all_module_timestamps,
        )
        .await;
    }

    if table_options.do_table_items() {
        debug!(
            "[Tx {}] Publishing {} tableitem records",
            cur_version,
            all_table_item_records.len()
        );

        publish_records(
            &publisher.table_items,
            all_table_item_records,
            Some(&format!("{}_{}", start, end)),
            all_table_item_timestamps,
        )
        .await;
    }

    if table_options.do_resources() {
        debug!(
            "[Tx {}] Publishing {} resource records",
            cur_version,
            all_resource_records.len()
        );
        publish_records(
            &publisher.resources,
            all_resource_records,
            Some(&format!("{}_{}", start, end)),
            all_resource_timestamps,
        )
        .await;
    }

    if table_options.do_blocks() {
        debug!("Publishing block Record");
        publish_records(
            &publisher.blocks,
            all_block_records,
            Some(&format!("{}_{}", start, end)),
            all_block_timestamps,
        )
        .await;
    }

    if table_options.do_events() {
        debug!("Publishing event Records");
        publish_records(
            &publisher.events,
            all_event_records,
            Some(&format!("{}_{}", start, end)),
            all_event_timestamps,
        )
        .await;
    }

    if table_options.do_signatures() {
        debug!("Publishing Signature Records");
        publish_records(
            &publisher.signatures,
            all_signature_records,
            Some(&format!("{}_{}", start, end)),
            all_signature_timestamps,
        )
        .await;
    }

    info!("Shutting down...");

    Ok(())
}

/// Given a [PathBuf] pointing to a file containing a [aptos_protos::transaction::v1::Transaction] as
/// a serialized protobuf.
pub fn load_tx(path: &PathBuf) -> Result<aptos_protos::transaction::v1::Transaction, ExtractError> {
    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut bytes: Vec<u8> = Vec::new();
            // Attempt to read in the bytes
            match file.read_to_end(&mut bytes) {
                Ok(bytelen) => debug!("Read in {:?}: {} bytes", path, bytelen),
                Err(error) => {
                    error!("Failed to read in the {:?} file", path);
                    return Err(ExtractError::FailedToOpenFile(path.clone(), error));
                }
            }
            // Decode the bytes into a Transaction
            match aptos_protos::transaction::v1::Transaction::decode(&*bytes) {
                Ok(tx) => Ok(tx),
                Err(error) => {
                    error!(
                        "Failed to decode Transaction while loading `{:?}`: {:?}",
                        path, error
                    );
                    Err(ExtractError::ProtoDecodeError(error))
                }
            }
        }
        Err(error) => {
            error!("Failed to open file: {}", error);
            Err(ExtractError::FailedToOpenFile(path.clone(), error))
        }
    }
}

/// Saves a Records
pub async fn save_records(
    records: proto_codegen::aptos::records::Records,
    outfile: &PathBuf,
) -> Result<(), ExtractError> {
    let mut f = match File::create(outfile.clone()) {
        Ok(file) => file,
        Err(error) => {
            error!(
                "Failed to create file while creating `{:?}` due to error: {:?}",
                outfile, error
            );
            return Err(ExtractError::FailedToCreateFile(outfile.clone(), error));
        }
    };

    // Serialize the transaction
    let serialized_records = records.encode_to_vec();

    // Save to file
    if let Err(error) = f.write_all(&serialized_records) {
        error!("Failed to serialize records: {}", error);
        return Err(ExtractError::Other(Box::new(error)));
    };

    Ok(())
}

/// Loads records from a file
pub fn load_records(
    path: &PathBuf,
) -> Result<proto_codegen::aptos::records::Records, ExtractError> {
    match File::open(path.clone()) {
        Ok(mut file) => {
            let mut bytes: Vec<u8> = Vec::new();
            // Attempt to read in the bytes
            match file.read_to_end(&mut bytes) {
                Ok(bytelen) => debug!("Read in {:?}: {} bytes", path, bytelen),
                Err(error) => {
                    error!("Failed to read in the {:?} file", path);
                    return Err(ExtractError::FailedToOpenFile(path.clone(), error));
                }
            }
            // Decode the bytes into a Transaction
            match proto_codegen::aptos::records::Records::decode(&*bytes) {
                Ok(tx) => Ok(tx),
                Err(error) => {
                    error!(
                        "Failed to decode Records while loading `{:?}`: {:?}",
                        path, error
                    );
                    Err(ExtractError::ProtoDecodeError(error))
                }
            }
        }
        Err(error) => {
            error!("Failed to open file: {}", error);
            Err(ExtractError::FailedToOpenFile(path.clone(), error))
        }
    }
}

use processing::tables::all::extract_records;

/// Creates the test data
///
/// If a publisher is provided, will include the output
pub async fn create_test_data(
    start: u64,
    end: u64,
    dir: &PathBuf,
    publisher: Option<blockchain_generic::output::publish::StreamPublisher>,
) -> Result<(), ExtractError> {
    info!("Creating test data [{}, {}]", start, end);
    info!("Creating directories");
    match std::fs::create_dir_all(dir.clone()) {
        Ok(_) => info!("Created directory: {:?}", dir),
        Err(err) => panic!("Failed to create output dir `{:?}` due to: {}", dir, err),
    }
    let txdir = dir.join("txs");
    match std::fs::create_dir_all(txdir.clone()) {
        Ok(_) => info!("Created directory for txs: {:?}", txdir),
        Err(err) => panic!(
            "Failed to create output txs dir `{:?}` due to: {}",
            txdir, err
        ),
    }
    let recdir = dir.join("records");
    match std::fs::create_dir_all(recdir.clone()) {
        Ok(_) => info!("Created directory for records: {:?}", recdir),
        Err(err) => panic!(
            "Failed to create output records dir `{:?}` due to: {}",
            recdir, err
        ),
    }

    info!("Extracting transactions");
    let txs = match extract_txs(start, end, Some(txdir)).await {
        Ok(txs) => txs,
        Err(err) => {
            error!("Failed to extract: {:?}", err);
            return Err(err.extracterror);
        }
    };

    info!("Transforming transactions");
    let _records = match transform_txs(txs, Some(&recdir), publisher).await {
        Ok(records) => records,
        Err(err) => {
            error!("Error occured: {}", err);
            return Err(err);
        }
    };

    Ok(())
}
