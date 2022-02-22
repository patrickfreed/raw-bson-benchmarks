# Raw BSON Benchmarks

This repo contains some benchmarks demonstrating how to leverage the raw BSON API in the [`bson`](https://crates.io/crates/bson) crate to improve the performance of reads using the MongoDB Rust driver ([`mongodb`](https://crates.io/crates/mongodb)).

## Running the benchmarks

1. Start up a `mongod` on the default port (`localhost:27017`).
2. Run `cargo bench`
