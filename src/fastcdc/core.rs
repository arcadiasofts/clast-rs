use crate::fastcdc::Normal;
use crate::fastcdc::chunk::Chunk;
use crate::fastcdc::cut::find_cutpoint;
use crate::fastcdc::mask::Masks;
use bytes::BytesMut;
use std::io::Read;
use std::io::{self};

/// Lower limit for the `min_size` parameter.
pub const MIN_CHUNK_SIZE_MIN: usize = 64;
/// Upper limit for the `min_size` parameter.
pub const MIN_CHUNK_SIZE_MAX: usize = 1_048_576; // 1 MB

/// Lower limit for the `avg_size` parameter.
pub const AVG_CHUNK_SIZE_MIN: usize = 256;
/// Upper limit for the `avg_size` parameter.
pub const AVG_CHUNK_SIZE_MAX: usize = 4_194_304; // 4 MB

/// Lower limit for the `max_size` parameter.
pub const MAX_CHUNK_SIZE_MIN: usize = 1024;
/// Upper limit for the `max_size` parameter.
pub const MAX_CHUNK_SIZE_MAX: usize = 16_777_216; // 16 MB

/// A FastCDC chunker implementation.
pub struct FastCDC {
    min_size: usize,
    avg_size: usize,
    max_size: usize,
    masks: Masks,
}

impl FastCDC {
    ///
    /// Constructs a new `FastCDC` instance.
    ///
    /// ## Arguments
    ///
    /// * `min_size`: The minimum size of a chunk.
    /// * `avg_size`: The target average size of a chunk.
    /// * `max_size`: The maximum size of a chunk.
    /// * `normal`: The normalization level for chunk size distribution.
    ///
    /// ## Panics
    ///
    /// Panics if `min_size`, `avg_size`, or `max_size` are outside the allowed bounds,
    /// or if `min_size < avg_size < max_size` is not satisfied.
    ///
    /// * `min_size`: 64 ~ 1,048,576 (1 MB)
    /// * `avg_size`: 256 ~ 4,194,304 (4 MB)
    /// * `max_size`: 1,024 (1 KB) ~ 16,777,216 (16 MB)
    ///
    pub fn new(min_size: usize, avg_size: usize, max_size: usize, normal: Normal) -> Self {
        match Self::try_new(min_size, avg_size, max_size, normal) {
            Ok(instance) => instance,
            Err(e) => panic!("{}", e),
        }
    }

    ///
    /// Constructs a new `FastCDC` instance.
    /// Unlike `new`, this method returns a `Result` instead of panicking on invalid arguments.
    ///
    /// ## Arguments
    ///
    /// * `min_size`: The minimum size of a chunk.
    /// * `avg_size`: The target average size of a chunk.
    /// * `max_size`: The maximum size of a chunk.
    /// * `normal`: The normalization level for chunk size distribution.
    ///
    /// ## Errors
    ///
    /// Returns an `std::io::Error` with `ErrorKind::InvalidInput`
    /// if `min_size`, `avg_size`, or `max_size` are outside the allowed bounds,
    /// or if `min_size < avg_size < max_size` is not satisfied.
    ///
    /// * `min_size`: 64 ~ 1,048,576 (1 MB)
    /// * `avg_size`: 256 ~ 4,194,304 (4 MB)
    /// * `max_size`: 1,024 (1 KB) ~ 16,777,216 (16 MB)
    ///
    pub fn try_new(
        min_size: usize,
        avg_size: usize,
        max_size: usize,
        normal: Normal,
    ) -> io::Result<Self> {
        if !(MIN_CHUNK_SIZE_MIN..=MIN_CHUNK_SIZE_MAX).contains(&min_size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "min_size must be between {} and {}",
                    MIN_CHUNK_SIZE_MIN, MIN_CHUNK_SIZE_MAX
                ),
            ));
        }

        if !(AVG_CHUNK_SIZE_MIN..=AVG_CHUNK_SIZE_MAX).contains(&avg_size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "avg_size must be between {} and {}",
                    AVG_CHUNK_SIZE_MIN, AVG_CHUNK_SIZE_MAX
                ),
            ));
        }

        if !(MAX_CHUNK_SIZE_MIN..=MAX_CHUNK_SIZE_MAX).contains(&max_size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "max_size must be between {} and {}",
                    MAX_CHUNK_SIZE_MIN, MAX_CHUNK_SIZE_MAX
                ),
            ));
        }

        if !(min_size < avg_size && avg_size < max_size) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "must satisfy the condition: min_size < avg_size < max_size",
            ));
        }

        Ok(Self {
            min_size,
            avg_size,
            max_size,
            masks: Masks::new(avg_size, normal),
        })
    }

    ///
    /// Creates an iterator that yields chunks from the provided reader.
    ///
    /// ## Arguments
    ///
    /// * `reader`: The source to read data from (must implement `Read`).
    ///
    pub fn chunks<R: Read>(&self, reader: R) -> FastCDCIter<'_, R> {
        FastCDCIter {
            chunker: self,
            reader,
            buf: BytesMut::with_capacity(self.max_size),
            processed: 0,
            eof: false,
        }
    }
}

/// An iterator that yields `Chunk`s from a `Read` source.
pub struct FastCDCIter<'a, R: Read> {
    chunker: &'a FastCDC,
    reader: R,
    buf: BytesMut,
    processed: u64,
    eof: bool,
}

impl<'a, R: Read> Iterator for FastCDCIter<'a, R> {
    type Item = io::Result<Chunk>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.eof && self.buf.is_empty() {
            return None;
        }

        while !self.eof && self.buf.len() < self.chunker.max_size {
            let buf_len = self.buf.len();
            let needed = self.chunker.max_size - buf_len;

            self.buf.resize(buf_len + needed, 0);

            match self.reader.read(&mut self.buf[buf_len..]) {
                Ok(0) => {
                    self.eof = true;
                    self.buf.truncate(buf_len);
                    break;
                }
                Ok(n) => {
                    self.buf.truncate(buf_len + n);
                }
                Err(e) => {
                    self.buf.truncate(buf_len);
                    return Some(Err(e));
                }
            }
        }

        if self.buf.is_empty() {
            return None;
        }

        let scan_len = self.buf.len().min(self.chunker.max_size);
        let (fp_hash, cutpoint) = find_cutpoint(
            &self.buf[..scan_len],
            self.chunker.min_size,
            self.chunker.avg_size,
            self.chunker.max_size,
            self.chunker.masks.mask_s,
            self.chunker.masks.mask_s_ls,
            self.chunker.masks.mask_l,
            self.chunker.masks.mask_l_ls,
        );

        let data = self.buf.split_to(cutpoint).freeze();

        let chunk = Chunk {
            fp_hash,
            data,
            offset: self.processed,
            length: cutpoint,
        };

        self.processed += cutpoint as u64;

        Some(Ok(chunk))
    }
}

#[cfg(test)]
mod tests {
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
            Err(io::Error::new(io::ErrorKind::Other, "simulated read error"))
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
}
