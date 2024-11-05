# Aptos-ETL Extractor-Transformer

* This directory contains a Rust codebase with the `E` and `T`  portions in `ETL`. The `L` scripts can be found in `/aptos-etl/loader/`.
* For native build instructions, see: `/aptos-etl/scripts/build_extractor_transformer.sh`
* For a Dockerfile to build this code, see: `/aptos-etl/iac/extractor_transformer_dockerfile`

## What this does

* This program requests transaction data from an Aptos node using its gRPC interface. Then, it transforms the received data into BigQuery records (formatted as JSON files). These record files are then uploaded to GCS buckets (one bucket per BigQuery table).
* This code is single-threaded, and is intended to be run in multiple instances in a Kubernetes cluster for high throughput. To ensure that these separate instances do not extract the same Aptos transaction data, they are coordinated by the the Python script in `/aptos-etl/indexing_coordinator/publish_ranges.py`.
* The coordination works by having each instance of this Rust code pull a task from a Pub/Sub topic (these tasks are published by the Python script). Each task is a range of transaction numbers (called "versions" in Aptos), and these instances each request their unique assigned transactions from the Aptos node simultaneously.
* Each instance of this Rust code uses the __same subscription__ to the Pub/Sub topic, which ensures that each instance pulls a different task (this is called "competing consumers").

## System Requirements

### For compiling this code

#### Protocol Buffers compiler

On Debian-based systems like Ubuntu:

```shell
sudo apt-get install protobuf-compiler
```

#### Rust compiler `rustc` and `cargo`

On Linux-based systems, it is recommended to install these using `rustup`: <https://rustup.rs/>

#### C++ compiler `g++`

For Debian Distributions:

```shell
sudo apt-get install build-essential
```

#### OpenSSL

For Debian Distributions:

```shell
sudo apt-get install libssl-dev
```

#### PostgreSQL

```shell
sudo apt-get install libpq-dev
```

### For running the executable

* Authentication with GCP
* Access to the Aptos node's gRPC port (in other words, deploy this in the same network as the node if its gRPC port isn't public)
* Hardware requirements not yet known
* Have a `.env`, view the `.env.example`
* Exposed prometheus metrics port (configured as 4000 in .env.example) and exposed readiness and healthiness port for Kubernetes probes (8080 in .env.example)
  * The readiness route is `/ready` and the liveness route is `/healthz`.
* Must have `libpq5` installed (`sudo apt-get install libpq5` on debian systems)

## Compile / Build

Install libpq5, on debian systems it should be as follows:

```bash
sudo apt-get install libpq5
```

Compile in debug profile for development:

```bash
# binary found in ./target/debug/blockchain_etl_indexer
cargo build
```

Compile in release profile for deployment:

```bash
# binary found in ./target/release/blockchain_etl_indexer
cargo build --release
```

or, compile in release profile and install:

```bash
# binary can be run with command: `blockchain_etl_indexer`
cargo install --path .
```

## Logging

This codebase uses 3 levels of logging (all written to `stdout`):

1. `error`
2. `warn`
3. `info`

* To select one, set the `RUST_LOG` environment variable to one of these values.
  * For __deployment__, `warn` is recommended.
  * For debugging and development, `info` is recommended.
  * The default value if one isn't selected is `ERROR`

NOTE: the RUST_LOG variable is not read from the `.env` file like the rest of the environment variables listed in the next section.

NOTE 2: if running with `warn` logging, there will be no messages if deployed correctly. You can check the startup probe to confirm that it started up correctly.

NOTE 3: the only thing written to `stderr` are crashes / panics.

## Environment Variables

The Rust code expects the environment variables in a file named `.env` in this directory.
Please see the `.env.example` file for the environment variables used during development. Some important notes:

1. The `METRICS_PORT` variable is used to create a webserver that provides Prometheus metrics. The Dockerfile in `aptos-etl/iac/extractor_transformer_dockerfile` is currently configured to expose port 4000 for this. So make sure to modify both if you want to change this value.
2. The 8 `QUEUE_NAME_*` variables are the names of the GCS buckets used for each table.
    * For example, the bucket for `mainnet` `transactions` data is currently deployed in the `aptos-bq` project's `aptos_mainnet_transactions` bucket, while the bucket for `testnet` `signatures` data is deployed to `aptos_testnet_signatures`.

    * The examples in the `.env.example` file have replaced the network name with `NETWORK`.

3. The `GOOGLE_APPLICATION_CREDENTIALS` is the path to a key for authentication with GCP. Currently, this code only needs this for:
    * uploading files to GCS buckets,
    * subscribing to messages from a Google Pub/Sub subscription.
4. The `APTOS_GRPC_ADDRESS` is used to connect to the Aptos node's gRPC interface. By default, the Aptos node exposes port 50051 for gRPC.

IMPORTANT: if you are deploying this code for __mainnet__ data, then you will need to set the `APTOS_GRPC_ADDRESS` to the address of the __mainnet__ node. Likewise, if deploying this code for __testnet__, set this variable to the __testnet__ node's address.

## CLI &  How to Run

* The compiled program takes the following arguments:

```bash
index-subscription <SUBSCRIPTION NAME>
```

Currently, we have 2 subscriptions deployed, one for the mainnet pipeline, and one for the testnet pipeline:

1. `indexing-ranges-subscription-mainnet`
2. `indexing-ranges-subscription-testnet`

As an example, if the program was installed, you can run like this (and replace `NETWORK`):

```bash
blockchain_etl_indexer indexing-ranges-subscription-NETWORK
```
