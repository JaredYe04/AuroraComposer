//! Register, motif, drum, and legacy-ID alias rules.

use super::implemented::*;
use crate::rule::{HardRule, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    vec![
        reg_001_hard(),
        reg_002_hard(),
        cont_001_hard(),
        cont_002_hard(),
        drum_001_hard(),
        rhyt_010_hard(),
    ]
}

pub fn soft_rules() -> Vec<SoftRule> {
    vec![
        cont_001_soft_soft(),
        moti_001_soft(),
        drum_003_soft(),
        vled_001_soft(),
        vled_003_soft(),
        vled_010_soft(),
        rhyt_001_soft(),
        rhyt_005_soft(),
    ]
}
