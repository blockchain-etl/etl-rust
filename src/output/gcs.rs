//! This module contains implementation details for
//! StreamPublisherConnection when the `GOOGLE_CLOUD_STORAGE` feature is
//! enabled. This allows StreamPublisherConnection to
//! publish jsonl files to GCS.

use chrono::Timelike;
use chrono::{TimeZone, Utc};
use log::{error, info, warn};
use prost::Message;
use serde::Serialize;

use google_cloud_storage::client::google_cloud_auth::credentials::CredentialsFile; // can get a "similar names but distinct types" error if we import this from the google_cloud_auth crate with mismatched crate versions
use google_cloud_storage::client::{Client, ClientConfig};
use google_cloud_storage::http::objects::upload::{Media, UploadObjectRequest, UploadType};

use crate::blockchain_config::proto_codegen::aptos::common::UnixTimestamp;

use super::environment::*;
use super::publish::{StreamPublisherConnection, StreamPublisherConnectionClient};

/// Opens the connection to a JSONL file.
pub async fn connect(queue_env: &str) -> StreamPublisherConnection {
    let gcp_config = {
        match get_gcp_credentials_json_path() {
            Some(key_path) => {
                let cred_file = CredentialsFile::new_from_file(key_path.to_owned())
                    .await
                    .expect("GCP credentials file exists");
                // authenticate using the key file
                ClientConfig::default()
                    .with_credentials(cred_file)
                    .await
                    .unwrap()
            }
            None => ClientConfig::default().with_auth().await.unwrap(),
        }
    };

    let bucket_name = dotenvy::var(queue_env)
        .unwrap_or_else(|_| panic!("{} should exist in .env file", queue_env))
        .parse::<String>()
        .unwrap();

    // Attempt to create the client using the configuration from above
    let gcp_client = Client::new(gcp_config);

    // Return the created connection
    StreamPublisherConnection {
        client: StreamPublisherConnectionClient::GcsBucket(gcp_client),
        queue_name: bucket_name,
    }
}

impl StreamPublisherConnectionClient {
    /// Publish a prost message to the JSON file
    #[inline]
    pub async fn publish_batch<T: Serialize + Message>(
        &self,
        bucket: &str,
        name: &str,
        timestamps: Vec<UnixTimestamp>,
        msg_batch: Vec<T>,
    ) {
        assert!(timestamps.len() == msg_batch.len());
        if timestamps.is_empty() {
            // TODO: remove this later.
            info!("skipping empty record batch...");
            return;
        }
        let StreamPublisherConnectionClient::GcsBucket(gcs_client) = self;

        // Converts the records into strings
        let mut record_strings: Vec<String> = msg_batch
            .into_iter()
            .map(|record| serde_json::to_string::<T>(&record).unwrap())
            .collect();

        /*
        we have a vector of timestamps
        and a vector of records

        we need to batch up the records based on the timestamps.

        idea:
        1. we iterate through the timestamps (assuming they're in order of ascending timestamp)
        2. increment a counter of num_in_batch each time we encounter a timestamp that is in the same bounds
        3. when we encounter the next timestamp, we slice the previous batch up and publish them into the correct directory

         */

        #[allow(unused_assignments)]
        let mut directory_destination = String::new();
        let mut cur_dir = None;
        let mut prev_i = 0;
        let mut filename: Option<String> = None;
        for (i, ts) in timestamps.into_iter().enumerate() {
            filename = Some([name, "_", &prev_i.to_string(), ".jsonl"].concat());

            let dt = match Utc.timestamp_opt(ts.seconds, ts.nanos) {
                chrono::LocalResult::None => panic!("timestamp should be valid and parseable"),
                chrono::LocalResult::Single(s) => s,
                chrono::LocalResult::Ambiguous(_, _) => panic!("UTC doesn't have DST"),
            };

            directory_destination = if dt.minute() < 30 {
                let date = dt.date_naive().to_string();
                let hour = dt.hour().to_string();
                let minute = String::from("0");
                [date, hour, minute].join("/")
            } else {
                let date = dt.date_naive().to_string();
                let hour = dt.hour().to_string();
                let minute = String::from("30");
                [date, hour, minute].join("/")
            };

            match cur_dir {
                None => {}
                Some(ref c) => {
                    if c == &directory_destination {
                        continue;
                    } else {
                        // TODO: consider using a multi-part upload if the file content is really large
                        let c_string: String = String::from(c);
                        let file_destination = [c_string, filename.clone().unwrap()].join("/");
                        let upload_type = UploadType::Simple(Media::new(file_destination));
                        loop {
                            // if we want to use transactions at indices 0 and 1,
                            // then we need to split_off() with a value of 2. so if i=1 and prev_i=0, then 1-0+1 = 2.
                            let num_to_drain = i - prev_i;
                            let batch: Vec<String> =
                                record_strings.drain(0..num_to_drain).collect();
                            // Joins the record strings with a newline character between them
                            let concatenated_records: String = batch
                                .into_iter()
                                .reduce(|acc, record| acc + "\n" + &record)
                                .unwrap_or(String::new()); // create an empty file if there are no records
                            let uploaded = gcs_client
                                .upload_object(
                                    &UploadObjectRequest {
                                        bucket: bucket.to_owned(),
                                        ..Default::default()
                                    },
                                    concatenated_records.clone(),
                                    &upload_type,
                                )
                                .await;
                            match uploaded {
                                Ok(_) => break,
                                Err(e) => {
                                    error!("Failed to upload to GCS. Error: {:?}", e);
                                    warn!("Retrying GCS upload...");
                                    //panic!("Panicking due to failed GCS upload. Perhaps this should be uploaded as multi-part instead of simple? Or simply retried with backoff until success? Or maybe something has been misconfigured.");
                                }
                            }
                        }
                        prev_i = i;
                    }
                }
            }
            cur_dir = Some(directory_destination.clone());
        }
        let file_destination = [cur_dir.unwrap(), filename.unwrap()].join("/");
        let upload_type = UploadType::Simple(Media::new(file_destination));
        // Joins the record strings with a newline character between them
        let concatenated_records: String = record_strings
            .clone()
            .into_iter()
            .reduce(|acc, record| acc + "\n" + &record)
            .unwrap_or(String::new()); // create an empty file if there are no records
        loop {
            let uploaded = gcs_client
                .upload_object(
                    &UploadObjectRequest {
                        bucket: bucket.to_owned(),
                        ..Default::default()
                    },
                    concatenated_records.clone(),
                    &upload_type,
                )
                .await;
            match uploaded {
                Ok(_) => break,
                Err(e) => {
                    error!("Failed to upload to GCS. Error: {:?}", e);
                    warn!("Retrying GCS upload...");
                    //panic!("Panicking due to failed GCS upload. Perhaps this should be uploaded as multi-part instead of simple? Or simply retried with backoff until success? Or maybe something has been misconfigured.");
                }
            }
        }
    }

    /// Publish a prost message to the JSON file
    #[inline]
    pub async fn publish<T: Serialize + Message>(&self, bucket: &str, name: &str, msg: T) {
        let StreamPublisherConnectionClient::GcsBucket(gcs_client) = self;

        // TODO: try using serde_json::to_vec()
        let record_string = serde_json::to_string::<T>(&msg).unwrap();

        let filename = [name, ".json"].concat();

        let upload_type = UploadType::Simple(Media::new(filename));

        let uploaded = gcs_client
            .upload_object(
                &UploadObjectRequest {
                    bucket: bucket.to_owned(),
                    ..Default::default()
                },
                record_string,
                &upload_type,
            )
            .await;
        match uploaded {
            Ok(_) => (),
            Err(e) => {
                error!("Failed to upload to GCS. Error: {:?}", e);
                panic!("Panicking due to failed GCS upload. Perhaps this should be uploaded as multi-part instead of simple? Or simply retried with backoff until success? Or maybe something has been misconfigured.");
            }
        }
    }
}

impl StreamPublisherConnection {
    /// Publish a prost message to the JSON file
    #[inline]
    pub async fn publish_batch<T: Serialize + Message>(
        &self,
        filename: &str,
        timestamp: Vec<UnixTimestamp>,
        msg_batch: Vec<T>,
    ) {
        self.client
            .publish_batch(&self.queue_name, filename, timestamp, msg_batch)
            .await;
    }

    /// Publish a prost message to the JSON file
    #[inline]
    pub async fn publish<T: Serialize + Message>(&self, filename: &str, msg: T) {
        self.client.publish(&self.queue_name, filename, msg).await;
    }
}
