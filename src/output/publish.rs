//! This file stores output output struct that compile different functions
//! depending on the compilation features enabled.  The objective is to allow
//! the structs and functions provided in this module regardless of the comilation
//! features enabled.

#[cfg(feature = "SINGLE_PUBLISHER")]
pub use super::single_stream_publisher::StreamPublisher;

// This needs to be defined by the blockchain config due to using the names of the tables to identify each publisher.
#[cfg(feature = "SEPARATE_PUBLISHERS")]
pub use crate::blockchain_config::streampublisher::StreamPublisher;

// Get the appropriate connect
#[cfg(feature = "APACHE_KAFKA")]
pub use super::apache_kafka::connect;
#[cfg(feature = "GOOGLE_PUBSUB")]
pub use super::google_pubsub::connect;
#[cfg(feature = "JSON")]
pub use super::json::connect;
#[cfg(feature = "JSONL")]
pub use super::jsonl::connect;
#[cfg(feature = "RABBITMQ_CLASSIC")]
pub use super::rabbitmq_classic::connect;
#[cfg(feature = "RABBITMQ_STREAM")]
pub use super::rabbitmq_stream::connect;

/// An enum that represents a connection to an output.  Will only contain one item
/// dependent on the enabled features.
#[derive(Clone)]
pub enum StreamPublisherConnectionClient {
    #[cfg(feature = "GOOGLE_PUBSUB")]
    GcpPubSub(google_cloud_pubsub::publisher::Publisher),
    #[cfg(feature = "GOOGLE_CLOUD_STORAGE")]
    GcsBucket(google_cloud_storage::client::Client),
    #[cfg(feature = "APACHE_KAFKA")]
    ApacheKafka(
        std::sync::Arc<rskafka::client::partition::PartitionClient>, // /*rskafka::client::producer::BatchProducer<rskafka::client::producer::aggregator::RecordAggregator,>,
    ),
    #[cfg(feature = "RABBITMQ_CLASSIC")]
    RabbitMQClassic(amqprs::connection::Connection),
    #[cfg(feature = "RABBITMQ_STREAM")]
    RabbitMQStream(rabbitmq_stream_client::Producer<rabbitmq_stream_client::NoDedup>),
    #[cfg(feature = "JSONL")]
    JsonL(std::path::PathBuf),
    #[cfg(feature = "JSON")]
    Json(std::path::PathBuf),
}

/// A struct that contains the client used to connect to the publisher and the queue_name
//#[derive(Clone)]
pub struct StreamPublisherConnection {
    /// The `client` is an Enum with a singular item depending on the enabled features that
    /// contain the functionality of publishing
    pub client: StreamPublisherConnectionClient,
    /// The `queue_name` is a string to represent the output stream.  This would be things like
    /// the google pubsub topic, the rabbitmq queue or stream name, etc.
    pub queue_name: String,

    /// Not thread-safe. Needs to be constructed within the thread that is using it.
    #[cfg(feature = "RABBITMQ_CLASSIC")]
    pub channel: Option<amqprs::channel::Channel>,

    /// Not thread-safe. Needs to be constructed within the thread that is using it.
    #[cfg(feature = "APACHE_KAFKA")]
    pub producer: Option<
        rskafka::client::producer::BatchProducer<
            rskafka::client::producer::aggregator::RecordAggregator,
        >,
    >,
}

impl Clone for StreamPublisherConnection {
    fn clone(&self) -> StreamPublisherConnection {
        StreamPublisherConnection {
            client: self.client.clone(),
            queue_name: self.queue_name.clone(),
            #[cfg(feature = "RABBITMQ_CLASSIC")]
            channel: None,
            #[cfg(feature = "APACHE_KAFKA")]
            producer: None,
        }
    }
}
