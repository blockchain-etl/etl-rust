# ETL Rust (was etl-core)

This repository serves as the basis for a modular blockchain indexing framework. The primary purpose of this is to serve data for Google BigQuery, but outputs for Google Pub/Sub, RabbitMQ, RabbitMQ Stream, JSON files, and JSONL files are supported. In the future, support for more output types could be added in `src/output/`.

This repository contains generic code that should apply to any blockchain with a JSON RPC API, and has the concept of block indices. In the future, to support other API types, new request code could be added in `src/source/`.

Rust's feature system is used to conditionally compile the output type and blockchain-specific code.

All blockchain-specific details (HTTP request and response formats, indexing algorithm, and other implementation details) belong in their own subdirectories in `src/`, following the naming format of `BLOCKCHAIN-config`. For an example, see [solana-etl](https://github.com/blockchain-etl/solana-etl), which uses this framework.

For more information, please check the [documentation](/docs/).

# Setup
Use the script in `scripts/setup.sh` to automatically install system dependencies, clone the repo and all submodules, and compile:
-  Tested on Ubuntu LTS 22.04
```
bash scripts/setup.sh
```
NOTE: you may need to run with `sudo`.

Next, build and run the development profile (default) with appropriate features:

E.g. to index Solana and output to Google Pub/Sub, replace `<BLOCKCHAIN>` with `SOLANA` and `<OUTPUT_TYPE>` with `GOOGLE_PUBSUB`:

`cargo build --features <BLOCKCHAIN>,<OUTPUT_TYPE>`

Finally, execute with the appropriate function and parameters.

E.g. Index starting from genesis onwards:

`./target/debug/blockchain_etl_indexer index-range stream 0`

Or to index from genesis to block 10:
`./target/debug/blockchain_etl_indexer index-range stream 0 10`

And to index a list of specific blocks, provide a CSV filepath with `index-list` command:
`./target/debug/blockchain_etl_indexer index-list stream FILE_PATH.csv`
