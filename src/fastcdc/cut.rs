include!(concat!(env!("OUT_DIR"), "/gear_table.rs"));

///
/// Identifies the cut point (chunk boundary) within the buffer using the FastCDC algorithm,
/// with support for incremental scanning.
///
/// This function enables resuming the scanning process from a specific offset using a previously
/// computed hash value. This avoids re-scanning processed bytes, optimizing performance for
/// streaming or buffered data.
///
/// Returns a tuple containing the current rolling hash and the cut point offset.
///
/// ## Arguments
///
/// * `source`: The input data buffer to scan.
/// * `offset`: The byte offset to resume scanning from. Should be aligned to a 2-byte boundary.
/// * `prev_hash`: The rolling hash state at the given `offset`.
/// * `min_size`: The minimum allowed chunk size.
/// * `avg_size`: The target average chunk size.
/// * `max_size`: The maximum allowed chunk size.
/// * `mask_s`: Bitmask for the region smaller than the average size.
/// * `mask_s_ls`: Left-shifted version of `mask_s`.
/// * `mask_l`: Bitmask for the region larger than the average size.
/// * `mask_l_ls`: Left-shifted version of `mask_l`.
///
#[allow(clippy::too_many_arguments)]
#[inline]
pub(super) fn find_cutpoint_inner(
    source: &[u8],
    offset: usize,
    prev_hash: u64,
    min_size: usize,
    avg_size: usize,
    max_size: usize,
    mask_s: u64,
    mask_s_ls: u64,
    mask_l: u64,
    mask_l_ls: u64,
) -> (u64, usize) {
    let scan_len = source.len().min(max_size);

    if scan_len <= min_size {
        return (prev_hash, scan_len);
    }

    let (mut start_idx, mut fp_hash) = if offset < min_size {
        ((min_size / 2), 0u64)
    } else {
        let aligned_offset = (offset / 2) * 2;
        ((aligned_offset / 2), prev_hash)
    };

    let center_idx = avg_size.min(scan_len) / 2;
    let end_idx = scan_len / 2;

    if start_idx < center_idx {
        for pair_idx in start_idx..center_idx {
            let byte_idx = pair_idx * 2;

            if byte_idx + 1 >= source.len() {
                break;
            }

            fp_hash = (fp_hash << 2).wrapping_add(GEAR_LS[source[byte_idx] as usize]);

            if (fp_hash & mask_s_ls) == 0 {
                return (fp_hash, byte_idx);
            }

            fp_hash = fp_hash.wrapping_add(GEAR[source[byte_idx + 1] as usize]);

            if (fp_hash & mask_s) == 0 {
                return (fp_hash, byte_idx + 1);
            }
        }
        start_idx = center_idx;
    }

    start_idx = start_idx.max(center_idx);

    for pair_idx in start_idx..end_idx {
        let byte_idx = pair_idx * 2;

        if byte_idx + 1 >= source.len() {
            break;
        }

        fp_hash = (fp_hash << 2).wrapping_add(GEAR_LS[source[byte_idx] as usize]);

        if (fp_hash & mask_l_ls) == 0 {
            return (fp_hash, byte_idx);
        }

        fp_hash = fp_hash.wrapping_add(GEAR[source[byte_idx + 1] as usize]);

        if (fp_hash & mask_l) == 0 {
            return (fp_hash, byte_idx + 1);
        }
    }

    (fp_hash, scan_len)
}
