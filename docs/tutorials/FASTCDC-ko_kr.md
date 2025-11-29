# FastCDC 튜토리얼

**FastCDC**는 데이터를 가변 크기의 청크로 분할하는 콘텐츠 정의 청킹(Content-Defined Chunking) 알고리즘입니다.

## 사용법

### 동기(Synchronous) 예제

```rust
use clast::fastcdc::{FastCDC, Normal};
use std::io::Cursor;

fn main() -> std::io::Result<()> {
    let data = b"Hello, world! This is a test for Content-Defined Chunking.";
    let reader = Cursor::new(data);

    // 특정 파라미터로 청커 초기화
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

### 비동기(Asynchronous) 예제

`async` 기능 활성화가 필요합니다.

```rust
use clast::fastcdc::{FastCDC, Normal};
use futures::StreamExt;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let data = b"Hello, world! This is a test for Content-Defined Chunking.";
    let reader = &data[..]; // AsyncRead 구현

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

