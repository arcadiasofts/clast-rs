use rand_chacha::{
    ChaCha20Rng,
    rand_core::{RngCore, SeedableRng},
};
use std::{
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
        generate_gear_table();
    }
}

// --- Gear Table Generator ---

const DEFAULT_GEAR_SEED: u64 = 14387234659234864480;

fn generate_gear_table() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gear_table.rs");
    let mut file_buf = BufWriter::new(File::create(&dest_path).unwrap());

    let seed = env::var("GEAR_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_GEAR_SEED);

    let mut rng = ChaCha20Rng::seed_from_u64(seed);

    let mut gear_table = [0u64; 256];
    for val in gear_table.iter_mut() {
        *val = rng.next_u64();
    }

    // Write Gear Table
    writeln!(file_buf, "pub const GEAR: [u64; 256] = [").unwrap();

    for &val in gear_table.iter() {
        writeln!(file_buf, "    {:#018x},", val).unwrap();
    }

    writeln!(file_buf, "];").unwrap();

    writeln!(file_buf).unwrap();

    // Write Left-Shifted Gear Table
    writeln!(file_buf, "pub const GEAR_LS: [u64; 256] = [").unwrap();

    for &val in gear_table.iter() {
        writeln!(file_buf, "    {:#018x},", val << 1).unwrap();
    }

    writeln!(file_buf, "];").unwrap();
}
