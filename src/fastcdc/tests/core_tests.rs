use super::*;
use std::{env, fs, io, path::PathBuf};

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

#[test]
fn test_empty_input() {
    let data: [u8; 0] = [];
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut iter = chunker.chunks(&data[..]);

    // Empty input should not produce any chunks
    assert!(
        iter.next().is_none(),
        "Empty input should not yield any chunks"
    );
}

#[test]
fn test_small_input() {
    let data = generate_patterned_data(MIN_SIZE / 2);
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);
    let chunks = chunker
        .chunks(&data[..])
        .collect::<io::Result<Vec<_>>>()
        .expect("Failed to chunk small input");

    // Input smaller than min_size should result in a single chunk
    assert_eq!(
        chunks.len(),
        1,
        "Small input must produce exactly one chunk"
    );

    // The chunk length should match the source data length
    assert_eq!(chunks[0].length, data.len());

    // The chunk content should match the source data
    assert_eq!(chunks[0].data.as_ref(), &data[..]);
}

// --- Chunking Tests ---

#[test]
fn test_round_trip_chunking() {
    let data = generate_patterned_data(50_000);
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut reconstructed = Vec::with_capacity(data.len());
    let mut chunk_count = 0;

    for chunk in chunker.chunks(&data[..]) {
        let chunk = chunk.expect("Failed to read chunk");

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

#[test]
fn test_image_chunking() {
    let base_path = env!("CARGO_MANIFEST_DIR");
    let file_path = PathBuf::from(base_path).join("assets/test_image.jpg");

    if !file_path.exists() {
        eprintln!(
            "Test file not found at {:?}. Skipping image test.",
            file_path
        );
        return;
    }

    let file = fs::File::open(&file_path).expect("Failed to open test file");
    let file_len = file.metadata().expect("Failed to get file metadata").len() as usize;
    let reader = io::BufReader::new(file);

    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);

    let mut reconstructed = Vec::with_capacity(file_len);
    let mut total_len: usize = 0;

    for chunk in chunker.chunks(reader) {
        let chunk = chunk.expect("Failed to read chunk");

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
    // While this involves double IO, it ensures the reader interface works correctly while validating data integrity.
    let original_data = fs::read(&file_path).expect("Failed to read validation data");

    // Reconstructed data must be identical to the original file
    assert_eq!(
        reconstructed, original_data,
        "Reconstructed data does not match original file"
    );
}

// --- Error Test ---

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("simulated read error"))
    }
}

#[test]
fn test_reader_error() {
    let chunker = FastCDC::new(MIN_SIZE, AVG_SIZE, MAX_SIZE, Normal::Level2);
    let reader = FailingReader;

    let mut iter = chunker.chunks(reader);
    let result = iter.next().expect("Iterator expected to yield a result");

    // Verify that the iterator correctly propagates the error from the underlying reader
    assert!(
        result.is_err(),
        "Iterator failed to propagate the read error immediately"
    );
}
