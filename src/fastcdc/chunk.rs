use bytes::Bytes;

/// Represents a content-defined chunk.
#[derive(Debug)]
pub struct Chunk {
    /// The fingerprint (Gear Hash) of the chunk.
    pub fp_hash: u64,
    /// The actual chunk data.
    pub data: Bytes,
    /// The absolute offset of the chunk in the source stream.
    pub offset: u64,
    /// The length of the chunk in bytes.
    pub length: usize,
}
