# FastCDC 튜토리얼

**FastCDC**는 데이터를 가변 크기의 청크로 분할하는 콘텐츠 정의 청킹(Content-Defined Chunking) 알고리즘입니다.

<br/>

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

<br/>

## 빌드 설정

### Gear 테이블 생성 사용자 정의

FastCDC는 롤링 해시 계산을 위해 미리 계산된 Gear 테이블을 사용합니다. 기본적으로 이 테이블은 고정된 시드(`318046`)를 사용하여 생성됩니다. `GEAR_SEED` 환경 변수를 설정하여 빌드 시점에 이 시드를 변경할 수 있습니다.

이는 다른 해시 분포를 실험하거나 재현성을 보장하는 데 유용합니다.

```bash
# Linux / macOS
GEAR_SEED=12345 cargo build

# Windows (PowerShell)
$env:GEAR_SEED=12345; cargo build
```

