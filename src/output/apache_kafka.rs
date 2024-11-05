//! This module contains implementation details for
//! StreamPublisherConnection when the `APACHE_KAFKA`
//! feature is enabled.  This allows StreamPublisherConnection
//! to connect and publish to Apache Kafka.

use super::environment::*;
use super::publish::{StreamPublisherConnection, StreamPublisherConnectionClient};
use chrono::Utc;
use log::{info, warn};
use prost::Message;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{self, Duration};
use tokio::time::sleep;

use rskafka::{
    client::{
        partition::UnknownTopicHandling,
        producer::{aggregator::RecordAggregator, BatchProducer, BatchProducerBuilder},
        ClientBuilder,
    },
    record::Record,
};

//use amqprs::channel::Channel;

/// Connects to Apache Kafka.
/// Expects the following parameters to be stored in the .env file:
/// - `KAFKA_ADDRESS`
/// - `KAFKA_PORT`
pub async fn connect(queue_name: &str) -> StreamPublisherConnection {
    // Extract necessary information from the .env from the queue
    let topic_name = dotenvy::var(queue_name)
        .unwrap_or_else(|_| panic!("{} should exist in .env file", queue_name))
        .parse::<String>()
        .unwrap();

    let address = get_kafka_addr();
    let port = get_kafka_port();
    let connection = format!("{}:{}", address, port);
    info!("Creating kafka environment...");
    let client = ClientBuilder::new(vec![connection]).build().await.unwrap();
    let partition_client = Arc::new(
        client
            .partition_client(topic_name.clone(), 0, UnknownTopicHandling::Retry)
            .await
            .unwrap(),
    );

    StreamPublisherConnection {
        client: StreamPublisherConnectionClient::ApacheKafka(partition_client),
        queue_name: topic_name,
        producer: None,
    }
}

/// creates a kafka record object using the bytes
fn prepare_message(serialized_message: Vec<u8>) -> Record {
    // some notes:
    // 1. we're setting the timestamp here, though it might be slightly better to set the timestamp earlier on. in reality, the difference in the timestamp would be a only a few milliseconds, if even that.
    // 2. we're not using any headers or key. but this could change in the future
    Record {
        key: None,
        value: Some(serialized_message),
        headers: BTreeMap::new(),
        timestamp: Utc::now(),
    }
}

/// Publishes a record to apache kafka.
/// Each time publishing fails, the sleep time is increased by 1 second.
async fn publish_with_backoff(publisher: &BatchProducer<RecordAggregator>, message: Record) {
    let mut pub_res = publisher.produce(message.clone()).await;
    let mut backoff = 0;
    while pub_res.is_err() {
        info!("Message publish result: {:?}", pub_res);
        match pub_res {
            Ok(_) => break,
            Err(_) => {
                warn!("publish failed for publisher: {:?}", publisher);
                let seconds = time::Duration::from_secs(backoff);
                sleep(seconds).await;
                backoff += 1;
                pub_res = publisher.produce(message.clone()).await;
            }
        }
    }
}

impl StreamPublisherConnection {
    pub async fn with_producer(self) -> StreamPublisherConnection {
        let StreamPublisherConnectionClient::ApacheKafka(inner_client) = self.client;
        let queue_name = self.queue_name;
        let this_inner_client = inner_client.clone();
        // Create a partition client with the builder
        let producer = BatchProducerBuilder::new(this_inner_client.clone())
            .with_linger(Duration::ZERO)
            .build(RecordAggregator::new(1024)); // NOTE: the official docs use 1024 in the usage example, but it is unclear if this was an arbitrary or meaningful decision on their part.
        StreamPublisherConnection {
            client: StreamPublisherConnectionClient::ApacheKafka(this_inner_client),
            queue_name,
            producer: Some(producer),
        }
    }

    /// Sends the message to the client
    pub async fn publish<T: Message>(&self, msg: T) {
        let serialized_msg = msg.encode_to_vec();
        let prepared_msg = prepare_message(serialized_msg);
        let producer = self.producer.as_ref().expect(
            "producer should have been constructed with StreamPublisherConnection.with_producer()",
        );
        publish_with_backoff(producer, prepared_msg).await;
    }
}
