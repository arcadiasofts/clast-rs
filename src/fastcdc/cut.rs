include!(concat!(env!("OUT_DIR"), "/gear_table.rs"));

///
/// Determines the cut point (chunk boundary) within the buffer using the FastCDC algorithm.
///
/// Returns a tuple containing the fingerprint hash and the cut point offset.
///
/// ## Arguments
///
/// * `source`: The input data buffer to scan.
/// * `min_size`: The minimum allowed chunk size.
/// * `avg_size`: The target average chunk size.
/// * `max_size`: The maximum allowed chunk size.
/// * `mask_s`: Mask for the section smaller than the average size.
/// * `mask_s_ls`: Left-shifted `mask_s`.
/// * `mask_l`: Mask for the section larger than the average size.
/// * `mask_l_ls`: Left-shifted `mask_l`.
///
#[allow(clippy::too_many_arguments)]
#[inline]
pub fn find_cutpoint(
    source: &[u8],
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
        return (0, scan_len);
    }

    let start_idx = min_size / 2;
    let center_idx = avg_size.min(scan_len) / 2;
    let end_idx = scan_len / 2;

    let mut fp_hash: u64 = 0;

    for pair_idx in start_idx..center_idx {
        let byte_idx = pair_idx * 2;

        fp_hash = (fp_hash << 2).wrapping_add(GEAR_LS[source[byte_idx] as usize]);

        if (fp_hash & mask_s_ls) == 0 {
            return (fp_hash, byte_idx);
        }

        fp_hash = fp_hash.wrapping_add(GEAR[source[byte_idx + 1] as usize]);

        if (fp_hash & mask_s) == 0 {
            return (fp_hash, byte_idx + 1);
        }
    }

    for pair_idx in center_idx..end_idx {
        let byte_idx = pair_idx * 2;

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
