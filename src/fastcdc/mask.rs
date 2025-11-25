include!(concat!(env!("OUT_DIR"), "/mask_table.rs"));

/// Levels of chunk size normalization.
#[derive(Debug, Clone, Copy)]
pub enum Normal {
    /// No normalization.
    None,
    /// Level 1 normalization.
    Level1,
    /// Level 2 normalization (recommended).
    Level2,
    /// Level 3 normalization.
    Level3,
}

impl Normal {
    fn offset(&self) -> u32 {
        match self {
            Normal::None => 0,
            Normal::Level1 => 1,
            Normal::Level2 => 2,
            Normal::Level3 => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Masks {
    pub mask_s: u64,
    pub mask_s_ls: u64,
    pub mask_l: u64,
    pub mask_l_ls: u64,
}

impl Masks {
    pub fn new(avg_size: usize, normal: Normal) -> Self {
        let bits = avg_size.ilog2();
        let offset = normal.offset();

        let mask_s = MASK_TABLE[(bits + offset) as usize];
        let mask_s_ls = mask_s << 1;

        let mask_l = MASK_TABLE[(bits - offset) as usize];
        let mask_l_ls = mask_l << 1;

        Self {
            mask_s,
            mask_s_ls,
            mask_l,
            mask_l_ls,
        }
    }
}
