# Data Extraction

All RPC requests are retried with backoff upon failure, with failures logged at the `warning` level.

Blocks are requested from the node by the `call_getBlock()` function.

The `call_getBlockHeight()` function requests the current block height.

The `call_getMultipleAccounts()` function requests account data for a list of pubkeys. These pubkeys come from the created accounts and token mints in the block data.

The blockchain configuration is expected to define the HTTP requests that these functions make in a `<BLOCKCHAIN_CONFIG>/types/request_types.rs` file. These requests should be specified using `struct`s called `BlockHeightRequest` and `BlockRequest`, and should implement `serde::Serialize`. It is recommended that you annotate the struct with `#[derive(serde::Serialize)]` to simplify this process and generate the code.
