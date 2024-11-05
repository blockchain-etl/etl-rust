# Core Code

This directory is a derivation from the `etl-core` indexer framework, providing CLI access and some
other features to wrap the core logic for transformation of the Aptos data.

See the parent directory for information on how to build and run the extractor_transformer.

The `aptos_config` directory contains the code specific to the Aptos ETL, with most the transformation
logic in the /processing directory, and the protos in the /proto_src directory.
