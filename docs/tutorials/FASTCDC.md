# FastCDC Tutorial

**FastCDC** is a content-defined chunking algorithm that splits data into variable-sized chunks.

## Usage

### Synchronous Example

```rust
use clast::fastcdc::{FastCDC, Normal};
use std::io::Cursor;

fn main() -> std::io::Result<()> {
    let data = b"Hello, world! This is a test for Content-Defined Chunking.";
    let reader = Cursor::new(data);

    // Initialize the chunker with specific parameters
    // (min_size, avg_size, max_size, normalization_level)
    let chunker = FastCDC::new(16, 32, 64, Normal::Level2);

    for chunk in chunker.chunks(reader) {
        let chunk = chunk?;
        println!(
            "Chunk: offset={}, length={}, hash={:x}",
            chunk.offset, chunk.length, chunk.hash
        );
    }
    Ok(())
}
```

### Asynchronous Example

Requires the `async` feature.

```rust
use clast::fastcdc::{FastCDC, Normal};
use futures::StreamExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let data = b"Hello, world! This is a test for Content-Defined Chunking.";
    let reader = &data[..]; // Implements AsyncRead

    let chunker = FastCDC::new(16, 32, 64, Normal::Level2);
    let mut stream = chunker.chunks_async(reader);

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res?;
        println!(
            "Chunk: offset={}, length={}, hash={:x}",
            chunk.offset, chunk.length, chunk.hash
        );
    }
    Ok(())
}
```
