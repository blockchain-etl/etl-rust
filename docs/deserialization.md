# Response Deserialization

To deserialize JSON responses from the blockchain node, we expect the blockchain configuration to specify the structure of the response in a Rust `struct` and annotate it with the `Deserialize` macro from the `serde` library. This macro generates deserialization code for the developer which eases development, but more importantly allows us to deserialize it with the `simd-json` library.

The `simd-json` library uses CPU vector extensions for accelerated JSON deserialization. Currently, the library supports x86 and ARM vector extensions, but falls back to standard deserialization if used on a system that doesn't support SIMD.   
* Since x86's AVX2 is 256-bit, while ARM's NEON is 128-bit, *you can expect best performance on x86*.
* This library is only used when compiled in the `release` profile, because its error messages are less descriptive. For development, it is recommended that you compile in debug mode (the default profile), which will use the `serde` deserializer, thus providing more descriptive errors.  