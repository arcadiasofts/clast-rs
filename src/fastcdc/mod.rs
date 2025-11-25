//!
//! This module implements the FastCDC algorithm, as described in the paper:
//! **"The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems"**.
//!
//! ## Reference
//! * **Title**: The Design of Fast Content-Defined Chunking for Data Deduplication Based Storage Systems
//! * **Authors**: Wen Xia, Xiangyu Zou, Hong Jiang, Yukun Zhou, Chuanyi Liu, Dan Feng, Yu Hua, Yuchong Hu, and Yucheng Zhang.
//! * **Journal**: IEEE Transactions on Parallel and Distributed Systems, Vol. 31, No. 9, September 2020.
//! * **DOI**: 10.1109/TPDS.2020.2984632
//!
//! ## Key Features
//! This implementation incorporates the five key optimizations proposed in the paper:
//! 1. **Gear-based Rolling Hashing**: Utilizes a gear-based rolling hash, which is significantly faster than Rabin fingerprinting.
//! 2. **Optimizing Hash Judgment**: Employs zero-padding and simplified conditional statements to accelerate boundary detection.
//! 3. **Sub-minimum Chunk Cut-Point Skipping**: Bypasses hash computation for data segments smaller than the minimum chunk size.
//! 4. **Normalized Chunking**: Normalizes chunk-size distribution to mitigate lower deduplication ratios resulting from cut-point skipping.
//! 5. **Rolling Two Bytes each time**: Processes two bytes per iteration to further minimize CPU overhead.
//!

mod chunk;
mod core;
mod cut;
mod mask;

pub use chunk::Chunk;
pub use core::FastCDC;
pub use cut::find_cutpoint;
pub use mask::Normal;
