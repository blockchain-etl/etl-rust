// Note:  Cargo automated documentation doesn't apply to the main.rs file.  To write documentation
// for the index of the wiki page, do so in the lib.rs file.

// I wish cargo-fmt sorted these such that all of the actix_web imports could be together...
#[cfg(feature = "ORCHESTRATED")]
use actix_web::{web, HttpResponse};
use clap::{Args, Parser, Subcommand};
use log::{error, info};
use std::error::Error;
#[cfg(not(feature = "ORCHESTRATED"))]
use std::fs::{create_dir, read_dir, File};
#[allow(unused_imports)]
use std::path::{Path, PathBuf};
#[cfg(any(feature = "METRICS", feature = "ORCHESTRATED"))]
use {
    actix_web::{get, App, HttpServer, Responder},
    actix_web_prom::PrometheusMetricsBuilder,
};
#[cfg(feature = "ORCHESTRATED")]
use {
    google_cloud_auth::credentials::CredentialsFile,
    google_cloud_pubsub::client::{Client, ClientConfig},
};

use blockchain_etl_indexer::{aptos_config::create_test_data, blockchain_config};
use blockchain_etl_indexer::{metrics::Metrics, output::publish::StreamPublisher};

#[cfg(not(feature = "ORCHESTRATED"))]
use std::io::{BufRead, BufReader};

/// The directory containing the examples test range
pub const TEST_EXAMPLE_DIRECTORY: &str = "./tests/examples";

#[cfg(feature = "SOLANA_BIGTABLE")]
use blockchain_etl_indexer::solana_config::data_sources::bigtable;

// CLI Parsing information
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract using the contents of a Google Pub/Sub message
    #[cfg(feature = "ORCHESTRATED")]
    IndexSubscription(IndexSubscriptionArgs),
    /// Extract blocks from a starting index
    #[cfg(not(feature = "ORCHESTRATED"))]
    IndexRange(IndexRangeArgs),
    /// Extract blocks from a list
    #[cfg(not(feature = "ORCHESTRATED"))]
    IndexList(IndexListArgs),
    /// Save range
    SaveRange(SaveRangeArgs),
    // Creates a test range
    CreateTestSet(CreateTestRangeArgs),
}

/// Arguments relating the the indexing of the crypto currency, particularly output,
/// start point, and direction (reverse)
#[cfg(feature = "ORCHESTRATED")]
#[derive(Args)]
struct IndexSubscriptionArgs {
    /// The pub/sub topic to subscribe to.
    subscription: String,
}

/// Arguments relating the the indexing of the crypto currency, particularly output,
/// start point, and direction (reverse)
#[cfg(not(feature = "ORCHESTRATED"))]
#[derive(Args)]
struct IndexRangeArgs {
    /// The slot to begin indexing from
    start: u64,
    /// The slot to stop indexing at
    end: Option<u64>,
    /// Index backwards towards the genesis block
    #[clap(long)] // Long flag format ('--reverse')
    reverse: bool,
}

#[derive(Args)]
struct SaveRangeArgs {
    /// The slot to begin indexing from
    start: u64,
    /// The slot to stop indexing at
    end: u64,
    /// Path to the output directory
    outdir: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct CreateTestRangeArgs {
    /// The slot to begin indexing from
    start: u64,
    /// The slot to stop indexing at
    end: u64,
    /// The name
    name: String,
    /// Path to the output directory
    dir: Option<PathBuf>,
}

/// Arguments relating the the indexing of the crypto currency, particularly output,
/// start point, and direction (reverse)
#[cfg(not(feature = "ORCHESTRATED"))]
#[derive(Args)]
struct IndexListArgs {
    /// The path to a list of blocks to index.
    list: String,
}

/// Returns Welcome message when accessing the base-url of the server
#[cfg(feature = "METRICS")]
#[get("/")]
async fn index() -> impl Responder {
    "Welcome to ETL Metrics Server."
}

/// Liveness check for kubernetes
#[cfg(feature = "ORCHESTRATED")]
async fn liveness_probe() -> impl Responder {
    HttpResponse::Ok().body("Alive")
}

/// Readiness check for kubernetes
#[cfg(feature = "ORCHESTRATED")]
async fn readiness_probe() -> impl Responder {
    HttpResponse::Ok().body("Ready")
}

/// Reads in a CSV of u64 values and returns an iterator over the values.
#[cfg(not(feature = "ORCHESTRATED"))]
pub fn read_block_list_csv(file_path: &Path) -> Box<dyn Iterator<Item = u64>> {
    // determine if the first line of the csv seems like a header
    let has_headers = {
        let file = File::open(file_path).expect("file exists");
        let mut buf_reader = BufReader::new(file);
        let mut first_line = String::new();
        buf_reader
            .read_line(&mut first_line)
            .expect("file is readable");
        first_line
            .split(',')
            .all(|field| field.parse::<u64>().is_err())
            && first_line.trim().parse::<u64>().is_err()
    };

    // create the csv reader with the apparent header setting
    let rdr = csv::ReaderBuilder::new()
        .has_headers(has_headers)
        .from_path(file_path)
        .expect("csv exists and is readable");

    // read the csv as string values
    let raw_records: Vec<csv::StringRecord> = rdr
        .into_records()
        .map(|rec_result| rec_result.unwrap())
        .collect();

    // convert the strings to integers
    let parsed_records: Vec<Vec<u64>> = raw_records
        .into_iter()
        .map(|record| {
            record
                .into_iter()
                .map(|val| val.parse::<u64>().expect("value in record is a u64"))
                .collect::<Vec<u64>>()
        })
        .collect();

    // flatten all records into a single vector
    let values: Vec<u64> = parsed_records.into_iter().flatten().collect();

    // return an iterator over the values
    let values_iter = values.into_iter();

    Box::new(values_iter)
}

/// Opens the directory of indexed block numbers and determine where to pick up from.
#[cfg(not(feature = "ORCHESTRATED"))]
pub fn pick_up_from_previous_range(
    start: u64,
    end: Option<u64>,
    is_reverse: bool,
) -> (u64, Option<u64>) {
    let mut range_start = start;
    let mut range_end = end;
    // check for the index logs to pick up from a terminated run.
    //      - create the directory if it doesn't exist.
    let indexed_blocks_dir = Path::new("./indexed_blocks/");
    if indexed_blocks_dir.exists() {
        let indexed_blocks_files = read_dir(indexed_blocks_dir).unwrap();
        // the order that the files are iterated through is platform-dependent,
        // so we treat it as an unsorted list.
        for file in indexed_blocks_files {
            let path = file.unwrap().path();
            let filename = path.file_name().unwrap().to_str().unwrap();

            if is_reverse {
                let previous_start: u64 = filename.parse().expect("file name is a u64");
                match range_end {
                    Some(e) => {
                        if previous_start <= e {
                            range_end = Some(previous_start - 1);
                        }
                    }
                    None => {
                        if previous_start <= range_start {
                            range_start = previous_start - 1;
                        }
                    }
                }
            } else {
                let previous_end: u64 = filename.parse().expect("file name is a u64");
                if previous_end >= range_start {
                    range_start = previous_end + 1;
                    info!("setting range_start to {range_start}");
                }

                // ensure that we don't continue indexing beyond the range end, if passed in.
                if let Some(e) = end {
                    if previous_end >= e {
                        panic!("This range has already been indexed. Stopping...");
                    }
                }
            }
        }
    } else {
        create_dir(indexed_blocks_dir).expect("filesystem is writable");
    }

    (range_start, range_end)
}

/// Main function for the ETL-Core code.  Performs the following startup-tasks:
/// - Setup the logging system
/// - Loads in the .env
/// - Set up the RequestBuilder (reuse the same client)
/// - Setup the Prometheus metrics system
/// - Setup the stream connection (whether it is pubsub, rabbitmq, etc)
/// The main function then proceeds to run the indexer (either the custom indexer or the crypto-specific extract_all)
#[tokio::main]
#[allow(non_snake_case)]
async fn main() -> Result<(), Box<dyn Error>> {
    // Set up the logger
    env_logger::init();

    // Loads the .env file, raises an error otherwise
    dotenvy::dotenv().expect(".env file is required");

    // Set up the RequestBuilder to be used in the ETL-Core code.
    // NOTE: the reqwest docs suggest reusing a single client, rather than using multiple
    // the endpoint and request headers will be the same for every request, so
    // we will clone this request builder, rather than constructing a new one every time.
    #[cfg(feature = "SOLANA")]
    let request_builder = {
        let endpoint = dotenvy::var("ENDPOINT")
            .expect("ENDPOINT should exist in .env file")
            .parse::<String>()
            .unwrap();
        let connection_timeout = std::time::Duration::from_secs(constants::CONNECTION_TIMEOUT);
        let client_builder = reqwest::Client::builder().connect_timeout(connection_timeout);

        let client = client_builder.build().unwrap();
        let headers = request::get_headers();
        client.post(endpoint).headers(headers)
    };

    let cli = Cli::parse();

    // metrics setup
    // - Reads in the metrics address and port from the .env file
    // - Sets up th prometheus metrics server
    #[cfg(feature = "METRICS")]
    let (metrics, metrics_srv_handle) = {
        let (metrics_address, metrics_port) = {
            // env:metrics_address = Address for connecting to the Prometheus server
            let metrics_address = "127.0.0.1";
            // env:metrics_port = Port for connecting to the Prometheus server
            let metrics_port = dotenvy::var("METRICS_PORT")
                .expect("METRICS_PORT should exist in .env file")
                .parse::<u16>()
                .unwrap();
            (metrics_address, metrics_port)
        };

        let prometheus = PrometheusMetricsBuilder::new("api")
            .endpoint("/metrics")
            .build()
            .unwrap();

        let request_count =
            prometheus::IntCounter::new("request_count", "Total number of requests for all APIs")
                .unwrap();
        let failed_request_count = prometheus::IntCounter::new(
            "failed_request_count",
            "Total number of request failures for all APIs",
        )
        .unwrap();
        prometheus
            .registry
            .register(Box::new(request_count.clone()))
            .unwrap();
        prometheus
            .registry
            .register(Box::new(failed_request_count.clone()))
            .unwrap();

        let srv = HttpServer::new(move || App::new().wrap(prometheus.clone()).service(index))
            .bind((metrics_address, metrics_port))?
            .run();

        let srv_handle = srv.handle();

        tokio::task::spawn(srv);

        let metrics = Metrics {
            request_count,
            failed_request_count,
        };
        (Some(metrics), srv_handle)
    };

    #[cfg(not(feature = "METRICS"))]
    let metrics = None;

    // Kubernetes needs to be able to make health checks, so we spawn web servers for this here.
    #[cfg(feature = "ORCHESTRATED")]
    let health_check_srv_handle = {
        let health_checks_port = dotenvy::var("HEALTH_CHECKS_PORT")
            .expect("HEALTH_CHECKS_PORT should exist in .env file")
            .parse::<String>()
            .unwrap();

        let health_checks_address = ["0.0.0.0:", &health_checks_port].concat();

        let srv = HttpServer::new(|| {
            App::new()
                .route("/healthz", web::get().to(liveness_probe))
                .route("/ready", web::get().to(readiness_probe))
        })
        .bind(health_checks_address)?
        .run();

        let srv_handle = srv.handle();
        tokio::task::spawn(srv);
        srv_handle
    };

    match cli.command {
        #[cfg(feature = "ORCHESTRATED")]
        Commands::IndexSubscription(args) => {
            let subscription_arg = args.subscription;

            let gcp_config = match dotenvy::var("GOOGLE_APPLICATION_CREDENTIALS") {
                Ok(env_var) => {
                    let key_path = env_var.parse::<String>().unwrap();
                    let cred_file = CredentialsFile::new_from_file(key_path.to_owned())
                        .await
                        .expect("GCP credentials file exists");
                    // authenticate using the key file
                    ClientConfig::default()
                        .with_credentials(cred_file)
                        .await
                        .unwrap()
                }
                Err(_) => ClientConfig::default().with_auth().await.unwrap(),
            };

            // Attempt to create the client using the configuration from above
            let gcp_client = Client::new(gcp_config).await.unwrap();
            let subscription = gcp_client.subscription(&subscription_arg);

            let publisher = StreamPublisher::new().await;

            let cur_publisher = publisher.clone();

            blockchain_config::subscribe_and_extract(subscription, cur_publisher, metrics)
                .await
                .unwrap();

            #[cfg(feature = "REQUIRES_DISCONNECT")]
            publisher.disconnect().await;
        }
        #[cfg(not(feature = "ORCHESTRATED"))]
        Commands::IndexRange(args) => {
            if args.start == 0 && args.end.is_none() && args.reverse {
                panic!("FATAL: cannot index backwards from genesis");
            }
            /*
                        let (start, opt_end) = pick_up_from_previous_range(args.start, args.end, args.reverse);

                        #[allow(clippy::collapsible_else_if)]
                        let indexing_range: Box<dyn Iterator<Item = u64>> = if args.reverse {
                            if let Some(end) = opt_end {
                                Box::new((start..end).rev())
                            } else {
                                Box::new((0..start).rev())
                            }
                        } else {
                            if let Some(end) = opt_end {
                                Box::new(start..end)
                            } else {
                                Box::new(start..)
                            }
                        };
            */
            let publisher = StreamPublisher::new().await;

            let cur_publisher = publisher.clone();

            blockchain_config::extract_range(
                args.start,
                args.end.unwrap(),
                cur_publisher,
                metrics,
                None,
            )
            .await
            .unwrap();

            #[cfg(feature = "REQUIRES_DISCONNECT")]
            publisher.disconnect().await;
        }
        #[cfg(not(feature = "ORCHESTRATED"))]
        Commands::IndexList(_) => {
            unreachable!("IndexList not supported")
        }
        Commands::SaveRange(args) => {
            match blockchain_config::extract_txs(args.start, args.end, Some(args.outdir.clone()))
                .await
            {
                Ok(_) => info!(
                    "Successfully saved [{}, {}] to {:?}",
                    args.start, args.end, args.outdir
                ),
                Err(error) => {
                    error!(
                        "Failed to save range [{},{}] due to error: {:?}",
                        args.start, args.end, error
                    );
                    panic!(
                        "Failed to save range [{},{}] due to error: {:?}",
                        args.start, args.end, error
                    );
                }
            };
        }
        Commands::CreateTestSet(args) => {
            let pdir = args.dir.unwrap_or(TEST_EXAMPLE_DIRECTORY.into());
            let dir = pdir.join(format!("{}_{}_{}", &args.name, &args.start, &args.end));

            match create_test_data(args.start, args.end, &dir, None).await {
                Ok(()) => info!("Created test data: {:?}", dir),
                Err(err) => error!("Failed to create test data: {}", err),
            }
        }
    }

    #[cfg(feature = "ORCHESTRATED")]
    health_check_srv_handle.stop(false).await;

    #[cfg(feature = "METRICS")]
    metrics_srv_handle.stop(false).await;

    Ok(())
}
