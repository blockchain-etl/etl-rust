# Features

## Synopsis

You can either define `--features` in the `Cargo.toml` file inside the `etl-rust` repository or specify them as part of a command.

`cargo build --features ARGS...`
`cargo run --features ARGS...`

The `--features` option is required to build or run the ETL project.

## Arguments

Currently, the following blockchains are supported:
- `SOLANA`

A message queue is required to be specified:
- `RABBITMQ` - a classic RabbitMQ queue
- `RABBITMQ_STREAM` - a RabbitMQ with Stream Queue plugin
- `GOOGLE_PUBSUB` - Google Cloud Pub/Sub
- `JSON` - separate JSON files for each record
- `JSONL` - separate JSONL files for all records in each table per block

## Examples

1. Build the local project and its dependencies for the _SOLANA_ blockchain and JSON exporter:
```
cargo build --release --features SOLANA,JSON
```

2. Run the local project and its dependencies for the _SOLANA_ blockchain and _RABBITMQ_STREAM_ exporter:
```
cargo run --features SOLANA,RABBITMQ_STREAM
```
