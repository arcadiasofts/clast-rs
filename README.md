<div align="center">

<img src="docs/banner.png" alt="Clast Banner"/>

<br/>

A high-performance, extensible **Content-Defined Chunking (CDC)** library for Rust.

<br/>

English │ [한국어](docs/README-ko_kr.md)

</div>

<br/>

## Overview

**Clast** is a modular library designed for **Content-Defined Chunking (CDC)**. It splits data into variable-sized chunks based on content, rather than fixed offsets (Fixed-Size Chunking, FSC), making it a critical building block for data deduplication, backup systems, and efficient storage solutions.

**Clast** is architected to support multiple chunking algorithms, allowing developers to choose the best strategy for their specific use cases.

<br/>

## Key Features

- **High Performance**: Optimized for throughput and low CPU overhead.
- **Modular Architecture**: Designed to support various CDC algorithms.
- **Async & Sync**: Support for both synchronous `std::io` and asynchronous `tokio` runtimes.

<br/>

## Supported Algorithms

### FastCDC
An implementation of the **FastCDC** algorithm as described in *[The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems](https://doi.org/10.1109/TPDS.2020.2984632)*.

It incorporates five key optimizations:
- Gear-based Rolling Hashing
- Optimized Hash Judgment
- Sub-minimum Chunk Cut-Point Skipping
- Normalized Chunking
- Rolling Two Bytes

<br/>

## Installation

Use `cargo` to install the package:

```bash
cargo add clast
```


### Feature Flags

**Clast** uses feature flags to minimize the compiled binary size. You can selectively enable the features you need.

- `fastcdc`: Enables the FastCDC algorithm implementation. (Enabled by default)
- `async`: Enables asynchronous support using `tokio`.

Example of enabling only `fastcdc` (default behavior):

```bash
cargo add clast
```

Example of enabling `fastcdc` and `async` support:

```bash
cargo add clast --features async
```

Or in your `Cargo.toml`:

```toml
[dependencies]
clast = { version = "1.0.3", features = ["async"] }
```

<br/>

## Usage

Please refer to the [Tutorials](docs/tutorials/MENU.md) for detailed usage examples.

<br/>

## Reference

* **FastCDC**: Wen Xia et al., "The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems," *IEEE Transactions on Parallel and Distributed Systems*, 2020.

<br/>

## Contributing

Contributions are welcome! Please feel free to open a Pull Request.

1. Fork the repository.
2. Create your feature branch (`git checkout -b feature/new-feature`).
3. Commit your changes (`git commit -m 'Add some feature'`).
4. Push to the branch (`git push origin feature/new-feature`).
5. Open a Pull Request.

Please make sure to run `cargo fmt` and `cargo test` before opening a Pull Request.

<br/>

## License

MIT © Arcadia Softs. See [LICENSE](LICENSE) for details.
