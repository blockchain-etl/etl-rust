# ETL Infrastructure Architecture

## Architecture Framework
This repository serves as the basis for a modular blockchain indexing framework. The primary purpose of this is to serve data for Google BigQuery, but outputs for Google Pub/Sub, RabbitMQ, RabbitMQ Stream, JSON files, and JSONL files are supported. In the future, support for more output types could be added in `src/output/`.

This repository contains generic code that should apply to any blockchain with a JSON RPC API, and has the concept of block indices. In the future, to support other API types, new request code could be added in `src/source/`.

Rust's feature system is used to conditionally compile the output type and blockchain-specific code.

All blockchain-specific details (HTTP request and response formats, indexing algorithm, and other implementation details) belong in their own subdirectories in `src/`, following the naming format of `BLOCKCHAIN-config`. For an example, see [solana-etl](https://github.com/blockchain-etl/solana-etl), which uses this framework.

## Macro Infrastructure
An RPC node is expected to serve requests. The blockchain-config is expected to define its indexing algorithm. For example, the Solana ETL continually requests a block at each slot value, then makes additional requests for new accounts and token mints. Upon response, the data is converted into a Protocol Buffers data format and sent to a streaming queue, such as Google Cloud Pub/Sub or RabbitMQ.

The detailed extraction process is explained in the [extraction](/docs/extraction.md) document.
