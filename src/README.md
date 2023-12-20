 ## ETL Infrastructure Architecture
 ### Architecture Framework
 The `etl-rust` repository serves as infrastructure for blockchain indexers, and can accept custom configurations - similar to a plugin system. Developers can set up configurations within `etl-rust/src/` in the format of `blockchain_config` (e.g. `solana_config`). As far as what kind of infrastructure is provided here, this repository currently provides code for a generalized CLI (accepting ranges of blocks or a list of specific blocks, and indexing forwards or backwards), making JSON RPC requests, and outputting records directly to Google Cloud Pub/Sub, RabbitMQ, or RabbitMQ Stream, or writing to JSON or JSONL files.

 Currently, the Solana ETL uses this infrastructure, and can be viewed as an example of how to use it, [here](https://github.com/blockchain-etl/solana-etl).

 ### Macro Infrastructure
 An RPC node is expected to serve requests. Blocks are continually requested using the node, and if necessary, other data such as accounts and tokens may be requested as well. Upon response, the data is converted into a Protocol Buffers data format and sent to a streaming queue, such as Google Cloud Pub/Sub or RabbitMQ or written to JSON or JSONL files.

 ## Response Deserialization
 To deserialize JSON responses from the blockchain node, the blockchain configuration is expected to specify the structure of the response in a Rust `struct` and annotate it with the `Deserialize` macro from the `serde` library. This macro generates deserialization code for the developer which eases development, but more importantly allows us to deserialize it with the `simd-json` library.

 The `simd-json` library uses CPU vector extensions for accelerated JSON deserialization. Currently, the library supports x86 and ARM vector extensions, but falls back to standard deserialization if used on a system that doesn't support SIMD.
 * Since x86's AVX2 is 256-bit, while ARM's NEON is 128-bit, *you can expect best performance on x86*.
 * This library is only used when compiled in the `release` profile, because its error messages are less descriptive. For development, it is recommended that you compile in debug mode (the default profile), which will use the `serde` deserializer, thus providing more descriptive errors.

 ## Environment Variables
 ### Synopsis

 You can define enviornment variables in a `.env` file. Examples are illustrated in `.env.example.`

 ### Variables
- `ENDPOINT`
 **Required**. Specifies the address to use for json RPC requests.

 - `FALLBACK_ENDPOINT`
 **Required**. Specifies the address to use for json RPC requests, when the primary endpoint is failing. This value can be the same `ENDPOINT`.

 - `NUM_EXTRACTOR_THREADS`
 **Required**. Specifies the number of concurrent threads to run an extract job.

 - `ENABLE_METRICS`
 **Required**. This variable determines whether to launch a metrics server to collect metrics for Prometheus.

 - `METRICS_ADDRESS`
 Optional. Required only if `ENABLE_METRICS` is true. Specifies the address of the metrics server.

 - `METRICS_PORT`
 Optional. Required only if `ENABLE_METRICS` is true. Specifies the port of the metrics server.

 - `RABBITMQ_ADDRESS`
 Optional. Required only if _STREAM_EXPORTER_  is set to `RABBITMQ_STREAM`. Specifies the address of RabbitMQ.

 - `RABBITMQ_PORT`
 Optional. Required only if _STREAM_EXPORTER_  is set to `RABBITMQ_STREAM`. Specifies the port of RabbitMQ.

 - `BIGTABLE_CRED`
 Optional. Specifies the file path of the credential file required to access GCP Bigtable.

 - `GCP_CREDENTIALS_JSON_PATH`
 Optional. Required only if _STREAM_EXPORTER_  is set to `GOOGLE_PUBSUB`. Specifies the file path of the credential file required to access Google Pubsub.

 - `GOOGLE_PUBSUB_TOPIC`
 Optional. Required only if _STREAM_EXPORTER_ is set to `GOOGLE_PUBSUB`. Specifies the Google Pubsub topic to be used during exporting. It is assumed that the PubSub Topic is already created.

 ## Data Extraction

 All RPC requests are retried with backoff upon failure, with failures logged at the `warning` level.

 Blocks are requested from the node by the `call_getBlock()` function.

 The `call_getBlockHeight()` function requests the current block height.

 The `call_getMultipleAccounts()` function requests account data for a list of pubkeys. These pubkeys come from the created accounts and token mints in the block data.

 The blockchain configuration is expected to define the HTTP requests that these functions make in a `<BLOCKCHAIN_CONFIG>/types/request_types.rs` file. These requests should be specified using `struct`s called `BlockHeightRequest` and `BlockRequest`, and should implement `serde::Serialize`. It is recommended that you annotate the struct with `#[derive(serde::Serialize)]`  to simplify this process and generate the code.

 ## Features

 ### Synopsis

 You can either define `--features` in the `Cargo.toml` file inside the `etl-rust` repository or specify them as part of the `cargo build` or `cargo run` command:

 `cargo build --features ARGS...`
 `cargo run --features ARGS...`

 The `--features` option is required to build or run the ETL project.

 ### Arguments

 Currently, the following blockchains are supported:
 - `SOLANA`

 An output type is required to be specified:
 - `RABBITMQ` - a classic RabbitMQ queue
 - `RABBITMQ_STREAM` - a RabbitMQ with Stream Queue plugin
 - `GOOGLE_PUBSUB` - Google Cloud Pub/Sub
 - `JSON` - separate JSON files for each record
 - `JSONL` - separate JSONL files for all records in each table per block

 ## Protocol Buffers

 We use protocol buffers to serialize our data for transmission to a pub/sub system like RabbitMQ or Google Cloud Pub/Sub.

 Some blockchains provide their own protobuf interfaces, so when possible, we will attempt to use those.

 ### Codegen
 To generate Rust code from our protobuf interface, we use the `PROST` library. This is a popular library for Rust, and is used by the Solana blockchain with their official "storage" protobuf. We perform this codegen at compile time, using a custom Rust build script: `build_proto.rs`. This script uses the `include!` macro to import the protobuf build script from the blockchain-specific configuration. It is expected that each blockchain config will define its own protobuf build script.
