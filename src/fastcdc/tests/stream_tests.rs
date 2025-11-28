use super::*;
use crate::fastcdc::Normal;
use futures::StreamExt;
use std::{env, fs, io, path::PathBuf};
use tokio::io::{AsyncRead, ReadBuf};

const MIN_SIZE: usize = 4_069;
const AVG_SIZE: usize = 8_192;
const MAX_SIZE: usize = 16_384;

fn generate_patterned_data(len: usize) -> Vec<u8> {
    const BLOCKS: [&[u8]; 3] = [b"LOREM", b"IPSUM", b"DOLOR"];

    let mut data = Vec::with_capacity(len);
    let mut idx = 0;

    while data.len() < len {
        data.extend_from_slice(BLOCKS[idx % BLOCKS.len()]);
        idx += 1;
    }

    data.truncate(len);
    data
}

// --- Input Tests ---

#[tokio::test]
async fn test_empty_input() {
    let data: [u8; 0] = [];
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut stream = chunker.as_stream(&data[..]);

    // Empty input should not produce any chunks
    assert!(
        stream.next().await.is_none(),
        "Empty input should not yield any chunks"
    );
}

#[tokio::test]
async fn test_small_input() {
    let data = generate_patterned_data(MIN_SIZE / 2);
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let chunks: Vec<_> = chunker.as_stream(&data[..]).collect::<Vec<_>>().await;

    // Input smaller than min_size should result in a single chunk
    assert_eq!(
        chunks.len(),
        1,
        "Small input must produce exactly one chunk"
    );

    let chunk = chunks[0].as_ref().expect("Failed to chunk small input");

    // The chunk length should match the source data length
    assert_eq!(chunk.length, data.len());

    // The chunk content should match the source data
    assert_eq!(chunk.data.as_ref(), &data[..]);
}

// --- Chunking Tests ---

#[tokio::test]
async fn test_round_trip_chunking() {
    let data = generate_patterned_data(50_000);
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut reconstructed = Vec::with_capacity(data.len());
    let mut chunk_count = 0;

    let mut stream = chunker.as_stream(&data[..]);

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.expect("Failed to read chunk");
        reconstructed.extend_from_slice(chunk.data.as_ref());
        chunk_count += 1;
    }

    // Must produce at least one chunk
    assert!(
        chunk_count > 0,
        "Input data should yield at least one chunk"
    );

    // Reconstructed data must match the original data
    assert_eq!(
        reconstructed, data,
        "Reconstructed data does not match original"
    );
}

#[tokio::test]
async fn test_image_chunking() {
    let base_path = env!("CARGO_MANIFEST_DIR");
    let file_path = PathBuf::from(base_path).join("test/test_image.jpg");

    if !file_path.exists() {
        eprintln!(
            "Test file not found at {:?}. Skipping image test.",
            file_path
        );
        return;
    }

    let file = fs::File::open(&file_path).expect("Failed to open test file");
    let file_len = file.metadata().expect("Failed to get file metadata").len() as usize;

    // Convert std::fs::File to tokio::fs::File for AsyncRead
    let file = tokio::fs::File::from_std(file);
    let reader = tokio::io::BufReader::new(file);

    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut reconstructed = Vec::with_capacity(file_len);
    let mut total_len: usize = 0;

    let mut stream = chunker.as_stream(reader);

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.expect("Failed to read chunk");

        // Chunk size must not exceed max_size
        assert!(
            chunk.length <= MAX_SIZE,
            "Chunk size {} exceeds max_size {}",
            chunk.length,
            MAX_SIZE
        );

        reconstructed.extend_from_slice(&chunk.data);
        total_len += chunk.length;
    }

    // Total length of chunks must match the original file size
    assert_eq!(
        total_len, file_len,
        "Total chunk length does not match original file size"
    );

    // Verify the content by reading the file again.
    let original_data = fs::read(&file_path).expect("Failed to read validation data");

    // Reconstructed data must be identical to the original file
    assert_eq!(
        reconstructed, original_data,
        "Reconstructed data does not match original file"
    );
}

// --- Error Test ---

struct FailingReader;

impl AsyncRead for FailingReader {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        _buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        std::task::Poll::Ready(Err(io::Error::other("simulated read error")))
    }
}

#[tokio::test]
async fn test_reader_error() {
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);
    let reader = FailingReader;

    let mut stream = chunker.as_stream(reader);
    let result = stream
        .next()
        .await
        .expect("Stream expected to yield a result");

    // Verify that the stream correctly propagates the error from the underlying reader
    assert!(
        result.is_err(),
        "Stream failed to propagate the read error immediately"
    );
}
