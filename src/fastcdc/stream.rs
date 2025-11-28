use crate::fastcdc::{Chunk, FastCDC, cut::find_cutpoint_inner};
use bytes::BytesMut;
use futures::Stream;
use std::{
    io,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, ReadBuf};

impl FastCDC {
    ///
    /// Creates a stream that yields chunks from the provided async reader.
    ///
    /// ## Arguments
    ///
    /// * `reader`: The source to read data from (must implement `AsyncRead`).
    ///
    pub fn as_stream<R>(&self, reader: R) -> FastCDCStream<'_, R>
    where
        R: AsyncRead + Unpin,
    {
        FastCDCStream {
            chunker: self,
            reader,
            buf: BytesMut::with_capacity(self.max_size),
            processed: 0,
            eof: false,
            scanned: 0,
            fp_hash: 0,
        }
    }

    #[inline]
    fn find_cutpoint_from(&self, source: &[u8], offset: usize, prev_hash: u64) -> (u64, usize) {
        find_cutpoint_inner(
            source,
            offset,
            prev_hash,
            self.min_size,
            self.avg_size,
            self.max_size,
            self.masks.mask_s,
            self.masks.mask_s_ls,
            self.masks.mask_l,
            self.masks.mask_l_ls,
        )
    }
}

pub struct FastCDCStream<'a, R>
where
    R: AsyncRead + Unpin,
{
    chunker: &'a FastCDC,
    reader: R,
    buf: BytesMut,
    processed: u64,
    eof: bool,
    scanned: usize,
    fp_hash: u64,
}

impl<'a, R> FastCDCStream<'a, R>
where
    R: AsyncRead + Unpin,
{
    fn yield_chunk(&mut self, cutpoint: usize, fp_hash: u64) -> Chunk {
        let data = self.buf.split_to(cutpoint).freeze();
        let chunk = Chunk {
            fp_hash,
            data,
            offset: self.processed,
            length: cutpoint,
        };

        self.processed += cutpoint as u64;
        self.scanned = 0;
        self.fp_hash = 0;

        chunk
    }
}

impl<'a, R> Stream for FastCDCStream<'a, R>
where
    R: AsyncRead + Unpin,
{
    type Item = io::Result<Chunk>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        loop {
            if this.eof && this.buf.is_empty() {
                return Poll::Ready(None);
            }

            if this.buf.len() >= this.chunker.min_size || (this.eof && !this.buf.is_empty()) {
                let scan_len = this.buf.len().min(this.chunker.max_size);

                // Resume search from `scanned` offset using saved `fp_hash` to ensure O(N) complexity.
                let (new_fp_hash, found_cutpoint) = this.chunker.find_cutpoint_from(
                    &this.buf[..scan_len],
                    this.scanned,
                    this.fp_hash,
                );

                let cutpoint = match found_cutpoint {
                    // A valid cutpoint found by the rolling hash.
                    cp if cp < scan_len => Some(cp),

                    // Force a cut if the buffer exceeds the maximum chunk size to prevent memory issues.
                    _ if this.buf.len() >= this.chunker.max_size => Some(this.chunker.max_size),

                    // Flush the remaining bytes as the last chunk if the stream has ended.
                    _ if this.eof => Some(scan_len),

                    // Return `None` to wait for more data if no conditions are met.
                    _ => None,
                };

                match cutpoint {
                    Some(cp) => {
                        let chunk = this.yield_chunk(cp, new_fp_hash);
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                    None => {
                        // Align cursor to 2-byte boundary and skip already checked bytes.
                        this.scanned = ((scan_len / 2) * 2).max(this.chunker.min_size);
                        this.fp_hash = new_fp_hash;
                    }
                }
            }

            if this.buf.len() < this.chunker.max_size && !this.eof {
                // Reserve space incrementally (4KB ~ remaining) to avoid large upfront allocation.
                let read_size = (4096)
                    .max(this.chunker.min_size)
                    .min(this.chunker.max_size.saturating_sub(this.buf.len()));
                if read_size > 0 {
                    this.buf.reserve(read_size);
                }

                let dst = this.buf.spare_capacity_mut();
                let mut read_buf = ReadBuf::uninit(dst);

                match Pin::new(&mut this.reader).poll_read(cx, &mut read_buf) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e))),
                    Poll::Ready(Ok(())) => {
                        let n = read_buf.filled().len();
                        if n == 0 {
                            this.eof = true;
                        } else {
                            // SAFETY: `read_buf` ensures `n` bytes were initialized/written.
                            unsafe {
                                let new_len = this.buf.len() + n;
                                this.buf.set_len(new_len);
                            }
                        }
                    }
                }
            } else {
                return Poll::Pending;
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/stream_tests.rs"]
mod tests;
