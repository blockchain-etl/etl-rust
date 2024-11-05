//! This file contains the streampublisher, a struct containing all the StreamPublisherConnections.
//! This is specific to blockchains since we have different outputs per blockchain.

// Conditional imports
use crate as blockchain_generic;

use blockchain_generic::output::publish::StreamPublisherConnection;
use log::info;

#[cfg(feature = "APACHE_KAFKA")]
use blockchain_generic::output::apache_kafka::connect;
#[cfg(feature = "GOOGLE_CLOUD_STORAGE")]
use blockchain_generic::output::gcs::connect;
#[cfg(feature = "GOOGLE_PUBSUB")]
use blockchain_generic::output::google_pubsub::connect;
#[cfg(feature = "JSON")]
use blockchain_generic::output::json::connect;
#[cfg(feature = "JSONL")]
use blockchain_generic::output::jsonl::connect;
#[cfg(feature = "RABBITMQ_CLASSIC")]
use blockchain_generic::output::rabbitmq_classic::connect;
#[cfg(feature = "RABBITMQ_STREAM")]
use blockchain_generic::output::rabbitmq_stream::connect;

/// StreamPublisher struct (seperate-publisher version) that contains various output
/// streams for different content.
#[cfg(feature = "SEPARATE_PUBLISHERS")]
#[derive(Clone)]
pub struct StreamPublisher {
    pub blocks: StreamPublisherConnection,
    pub transactions: StreamPublisherConnection,
    pub changes: StreamPublisherConnection,
    pub events: StreamPublisherConnection,
    pub modules: StreamPublisherConnection,
    pub resources: StreamPublisherConnection,
    pub signatures: StreamPublisherConnection,
    pub table_items: StreamPublisherConnection,
}

#[cfg(feature = "SEPARATE_PUBLISHERS")]
impl StreamPublisher {
    #[cfg(feature = "APACHE_KAFKA")]
    pub async fn with_producer(self) -> StreamPublisher {
        info!("Construction kafka producers...");
        StreamPublisher {
            blocks: self.blocks.with_producer().await,
            transactions: self.transactions.with_producer().await,
            changes: self.changes.with_producer().await,
            events: self.events.with_producer().await,
            modules: self.modules.with_producer().await,
            resources: self.resources.with_producer().await,
            signatures: self.signatures.with_producer().await,
            table_items: self.table_items.with_producer().await,
        }
    }

    #[cfg(feature = "RABBITMQ_CLASSIC")]
    pub async fn with_channel(self) -> StreamPublisher {
        info!("Construction rabbitmq channgels...");
        StreamPublisher {
            blocks: self.blocks.with_channel().await,
            transactions: self.transactions.with_channel().await,
            changes: self.changes.with_channel().await,
            events: self.events.with_channel().await,
            modules: self.modules.with_channel().await,
            resources: self.resources.with_channel().await,
            signatures: self.signatures.with_channel().await,
            table_items: self.table_items.with_channel().await,
        }
    }

    pub async fn new() -> StreamPublisher {
        info!("Connecting to the publishers...");
        StreamPublisher {
            blocks: connect("QUEUE_NAME_BLOCKS").await,
            transactions: connect("QUEUE_NAME_TRANSACTIONS").await,
            changes: connect("QUEUE_NAME_CHANGES").await,
            events: connect("QUEUE_NAME_EVENTS").await,
            modules: connect("QUEUE_NAME_MODULES").await,
            resources: connect("QUEUE_NAME_RESOURCES").await,
            signatures: connect("QUEUE_NAME_SIGNATURES").await,
            table_items: connect("QUEUE_NAME_TABLE_ITEMS").await,
        }
    }

    #[cfg(feature = "REQUIRES_DISCONNECT")]
    pub async fn disconnect(self) {
        info!("Disconnecting from publishers...");
        self.blocks.disconnect().await;
        self.transactions.disconnect().await;
        self.changes.disconnect().await;
        self.events.disconnect().await;
        self.modules.disconnect().await;
        self.resources.disconnect().await;
        self.signatures.disconnect().await;
        self.table_items.disconnect().await;
    }
}
