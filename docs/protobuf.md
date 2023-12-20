# Protocol Buffers

We use protocol buffers to serialize our data for transmission to a pub/sub system like RabbitMQ or Google Cloud Pub/Sub.

Some blockchains provide their own protobuf interfaces, so when possible, we will attempt to use those.

## Codegen
To generate Rust code from our protobuf interface, we use the `PROST` library. This is a popular library for Rust, and is used by the Solana blockchain with their official "storage" protobuf. We perform this codegen at compile time, using a custom Rust build script: `build_proto.rs`. This script uses the `include!` macro to import the protobuf build script from the blockchain-specific configuration. It is expected that each blockchain config will define its own protobuf build script. 