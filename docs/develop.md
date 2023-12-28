# Develop
## Install System Dependencies
Tested on Ubuntu LTS 22.04:
```
sudo apt install git cargo g++ protobuf-compiler
```
## Clone the Repo
```
git clone https://github.com/blockchain-etl/etl-rust.git
```
## Create a Configuration
The code in this repo will not compile as-is. Instead, you are expected to create a "configuration" in the `src` directory. As an example, you can see the Solana ETL and its configuration in the `solana_config` directory [here](https://github.com/blockchain-etl/solana-etl/tree/main/src/solana_config).

## Compile the Code
```
cd etl-rust
cargo build –-release --features <OUTPUT>,<CONFIG>
```
NOTES:
1. You should replace `<CONFIG>` in the above command with the configuration that you created (for example with Solana: `SOLANA`).
2. You must replace `<OUTPUT>` in the above command with one of the supported output types, depending on how you would like to run the indexer. The supported outputs are:
1. `JSON`
2. `JSONL`
3. `GOOGLE_PUBSUB`
4. `RABBITMQ_CLASSIC`
5. `RABBITMQ_STREAM`

If you would like to upload the records as files to GCS buckets, then you should use `JSONL` as output, and you can run [this script](/scripts/upload_to_gcs.sh) to continually upload them.

## Configure the Environment Variables
See the [documentation on environment variables](/docs/environment-variables.md).

## Run the Indexer
There are two CLI options to choose from:
1. `index-range`
2. `index-list`
Option 1 requires that you pass a starting slot to index from, and you can optionally provide a second slot as the ending index. The start is inclusive, and the end is exclusive.

Option 2 requires that you pass the path to a CSV file containing a list of specified slots to index.

As an example, if you would like to index from the genesis block onwards, you can run the following command:
```
RUST_LOG=WARN ./target/release/blockchain_etl_indexer index-range stream 0
```
NOTE: `RUST_LOG` specifies the logging level. For more information, see the `log` crate and [its logging levels](https://docs.rs/log/latest/log/enum.Level.html).
