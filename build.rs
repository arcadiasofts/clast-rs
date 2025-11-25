use rand_chacha::{
    ChaCha20Rng,
    rand_core::{RngCore, SeedableRng},
};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    // Fast CDC Feature
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_FASTCDC");

    if env::var("CARGO_FEATURE_FASTCDC").is_ok() {
        println!("cargo:rerun-if-env-changed=GEAR_SEED");
        let gear_table = generate_gear_table();
        generate_mask_table(&gear_table);
    }
}

// --- Gear Table Generator ---

const DEFAULT_GEAR_SEED: u64 = 14387234659234864480;
const GEAR_TABLE_SIZE: usize = 256;

fn generate_gear_table() -> [u64; GEAR_TABLE_SIZE] {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gear_table.rs");
    let mut file_buf = BufWriter::new(File::create(&dest_path).unwrap());

    let seed = env::var("GEAR_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_GEAR_SEED);

    let mut rng = ChaCha20Rng::seed_from_u64(seed);

    let mut gear_table = [0u64; GEAR_TABLE_SIZE];
    for val in gear_table.iter_mut() {
        *val = rng.next_u64();
    }

    // Write Gear Table
    writeln!(file_buf, "pub const GEAR: [u64; {}] = [", GEAR_TABLE_SIZE).unwrap();

    for &val in gear_table.iter() {
        writeln!(file_buf, "    {:#018x},", val).unwrap();
    }

    writeln!(file_buf, "];").unwrap();

    writeln!(file_buf).unwrap();

    // Write Left-Shifted Gear Table
    writeln!(
        file_buf,
        "pub const GEAR_LS: [u64; {}] = [",
        GEAR_TABLE_SIZE
    )
    .unwrap();

    for &val in gear_table.iter() {
        writeln!(file_buf, "    {:#018x},", val << 1).unwrap();
    }

    writeln!(file_buf, "];").unwrap();

    gear_table
}

// --- Mask Table Generator ---

const HIGH_BIT_RISK_START: usize = 60;
const MASK_TABLE_SIZE: usize = 26;
const MASK_PADDING_SLOTS: usize = 5;

fn generate_mask_table(gear_table: &[u64; GEAR_TABLE_SIZE]) {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("mask_table.rs");
    let mut file_buf = BufWriter::new(File::create(&dest_path).unwrap());

    writeln!(
        file_buf,
        "pub const MASK_TABLE: [u64; {}] = [",
        MASK_TABLE_SIZE
    )
    .unwrap();

    let stats = analyze_bits(gear_table);

    for idx in 0..MASK_TABLE_SIZE {
        if idx < MASK_PADDING_SLOTS {
            writeln!(file_buf, "    {:#x},", 0).unwrap();
            continue;
        }

        let bits_needed = idx;

        if bits_needed > stats.len() {
            panic!(
                "Not enough usable bits to generate mask for {} bits!",
                bits_needed
            );
        }

        let mask = find_optimal_mask(&stats, bits_needed);
        writeln!(file_buf, "    {:#x},", mask).unwrap();
    }

    writeln!(file_buf, "];").unwrap();
}

#[derive(Clone, Debug)]
struct BitStat {
    position: usize,
    bias: f64,
    column: Vec<f64>,
}

fn analyze_bits(gear_table: &[u64; GEAR_TABLE_SIZE]) -> Vec<BitStat> {
    let mut stats = Vec::new();

    for bit_pos in 0..64 {
        let is_high_bit_risk = bit_pos >= HIGH_BIT_RISK_START;

        let mut ones_count = 0;
        let mut raw_column = Vec::with_capacity(GEAR_TABLE_SIZE);

        for val in gear_table {
            let bit = (val >> bit_pos) & 1;
            raw_column.push(bit as f64);
            if bit == 1 {
                ones_count += 1;
            }
        }

        let raw_bias = (ones_count as f64 - 128.0).abs();

        let effective_bias = if is_high_bit_risk {
            raw_bias + 500.0
        } else {
            raw_bias
        };

        stats.push(BitStat {
            position: bit_pos,
            bias: effective_bias,
            column: raw_column,
        });
    }
    stats
}

fn find_optimal_mask(stats: &[BitStat], bits_needed: usize) -> u64 {
    let mut candidates = stats.to_vec();
    candidates.sort_by(|a, b| a.bias.partial_cmp(&b.bias).unwrap());

    let mut stat_map: HashMap<usize, &BitStat> = HashMap::with_capacity(stats.len());
    for stat in stats {
        stat_map.insert(stat.position, stat);
    }

    let mut selected_positions: HashSet<usize> = HashSet::with_capacity(bits_needed);
    let mut mask: u64 = 0;

    if let Some(first) = candidates.first() {
        selected_positions.insert(first.position);
        mask |= 1 << first.position;
    }

    while selected_positions.len() < bits_needed {
        let mut best_bit_pos = None;
        let mut min_score = f64::MAX;

        for candidate in &candidates {
            if selected_positions.contains(&candidate.position) {
                continue;
            }

            let mut correlation_sum = 0.0;
            for &selected_pos in &selected_positions {
                let selected_stat = stat_map.get(&selected_pos).unwrap();
                let corr = pearson_correlation(&candidate.column, &selected_stat.column);
                correlation_sum += corr.abs();
            }

            let score = correlation_sum + (candidate.bias * 0.02);

            if score < min_score {
                min_score = score;
                best_bit_pos = Some(candidate.position);
            }
        }

        if let Some(bit_pos) = best_bit_pos {
            selected_positions.insert(bit_pos);
            mask |= 1 << bit_pos;
        } else {
            break;
        }
    }

    mask
}

fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    let n = x.len() as f64;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_sq_x = 0.0;
    let mut sum_sq_y = 0.0;

    for i in 0..x.len() {
        sum_x += x[i];
        sum_y += y[i];
        sum_xy += x[i] * y[i];
        sum_sq_x += x[i] * x[i];
        sum_sq_y += y[i] * y[i];
    }

    let denominator = ((n * sum_sq_x - sum_x * sum_x) * (n * sum_sq_y - sum_y * sum_y)).sqrt();

    if denominator == 0.0 {
        0.0
    } else {
        (n * sum_xy - sum_x * sum_y) / denominator
    }
}
