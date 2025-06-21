# ttrpc-rs-benchmark

This repository contains a minimal microbenchmark to measure latency overhead
of [ttrpc-rust](https://github.com/containerd/ttrpc-rust), a lightweight and
efficient transport protocol based on gRPC for container communication.

## Features

- Measures round-trip latency of `ttrpc` client/server on:
  - **Unix domain sockets**
  - **TCP sockets** (loopback)
- Simple protobuf-based request/response

## Requirements

- Rust (stable or nightly)
- `protoc` (Protocol Buffers compiler)
- `protoc-gen-prost` plugin (optional, if regenerating protobufs)

## Build

```bash
cargo build --release
```

## Run

Run the benchmark:

```bash
./target/release/ttrpc_benchmark
```
## File Layout

- `src/`
  - `main.rs` — Benchmark logic
- `echo.proto` — Protobuf definitions
- `build.rs` — Generates Rust types from protobuf
- `Cargo.toml` — Dependencies and build config

## Sample Output

```bash
Running ttrpc-rust latency benchmark with 1000 iterations...

Testing Unix sockets...
Unix Socket Results:
  Min:     58.029µs
  Average: 73.217µs
  Max:     887.728µs
  P99:     116.979µs

Testing TCP sockets...
TCP Socket Results:
  Min:     81.40514ms
  Average: 82.004085ms
  Max:     83.021974ms
  P99:     82.465309ms

Comparison:
  Unix sockets are 1120.01x faster than TCP
```

## Optionally combine with `nodelay`

Disable Nagle's Algorithm: Use https://github.com/nubificus/nodelay to force `TCP_NODELAY`:

```bash
git clone https://github.com/nubificus/nodelay
cd nodelay
make
```

This would produce a shared object: `nodelay.so` which you can use with `LD_PRELOAD`:

```console
$ LD_PRELOAD=../nodelay/nodelay.so ./target/release/ttrpc-benchmark
Running ttrpc-rust latency benchmark with 1000 iterations...

Testing Unix sockets...
Unix Socket Results:
  Min:     55.985µs
  Average: 69.387µs
  Max:     373.772µs
  P99:     101.441µs

Testing TCP sockets...
[hook] TCP_NODELAY enabled on socket 16
[hook] TCP_NODELAY enabled on socket 12
TCP Socket Results:
  Min:     81.323µs
  Average: 92.432µs
  Max:     420.38µs
  P99:     126.688µs

Comparison:
  Unix sockets are 1.33x faster than TCP
```

## License

Licensed under [Apache 2.0](LICENSE).
