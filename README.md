# ETL Rust (was etl-core)

This repository serves as the basis for a modular blockchain indexing framework. The primary purpose of this is to serve data for Google BigQuery, but outputs for Google Pub/Sub, RabbitMQ, RabbitMQ Stream, JSON files, and JSONL files are supported. In the future, support for more output types could be added in `src/output/`.

This repository contains generic code that should apply to any blockchain with a JSON RPC API, and has the concept of block indices. In the future, to support other API types, new request code could be added in `src/source/`.

Rust's feature system is used to conditionally compile the output type and blockchain-specific code.

All blockchain-specific details (HTTP request and response formats, indexing algorithm, and other implementation details) belong in their own subdirectories in `src/`, following the naming format of `BLOCKCHAIN-config`. For an example, see [solana-etl](https://github.com/blockchain-etl/solana-etl), which uses this framework.

For more information, please check the [documentation](/docs/).

# Develop
This repository is intended to provide code for ETL developers; the code in this repo will not compile on its own. Instead, you will need to create a blockchain configuration in the `src` directory. See [the documentation](/docs/develop.md) for more details.
