#![doc = include_str!("README.md")]
pub mod publish;

#[cfg(feature = "SINGLE_PUBLISHER")]
pub mod single_stream_publisher;

#[cfg(feature = "GOOGLE_PUBSUB")]
pub mod google_pubsub;

#[cfg(feature = "APACHE_KAFKA")]
pub mod apache_kafka;

#[cfg(feature = "RABBITMQ_CLASSIC")]
pub mod rabbitmq_classic;

#[cfg(feature = "RABBITMQ_STREAM")]
pub mod rabbitmq_stream;

#[cfg(feature = "JSONL")]
pub mod jsonl;

#[cfg(feature = "JSON")]
pub mod json;

#[cfg(feature = "GOOGLE_CLOUD_STORAGE")]
pub mod gcs;

pub mod environment;

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_publisher_connection() {
        let _ = crate::output::publish::StreamPublisher::new().await;
        //publisher.disconnect().await;
    }
}
