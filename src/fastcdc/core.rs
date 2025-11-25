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
    /// Panics if `min_size`, `avg_size`, or `max_size` are outside the allowed bounds.
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
    /// if `min_size`, `avg_size`, or `max_size` are outside the allowed bounds.
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
