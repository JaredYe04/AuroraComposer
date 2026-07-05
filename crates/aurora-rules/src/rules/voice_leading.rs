//! Voice Leading rules from `docs/03-theory/voice-leading.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(vl_dbl_002_hard());
    rules.push(stub_hard("VL-DBL-003", "VL-DBL-003", RuleCategory::VoiceLeading));
    rules.push(stub_hard("VL-DBL-004", "VL-DBL-004", RuleCategory::VoiceLeading));
    rules.push(vl_mot_010_hard());
    rules.push(vl_rng_001_hard());
    rules.push(vl_rng_002_hard());
    rules.push(stub_hard("VL-RNG-003", "VL-RNG-003", RuleCategory::VoiceLeading));
    rules.push(stub_hard("VL-X-004", "VL-X-004", RuleCategory::VoiceLeading));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(vl_ct_001_soft());
    rules.push(stub_soft("VL-CT-002", "VL-CT-002", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-CT-003", "VL-CT-003", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-CT-004", "VL-CT-004", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-CT-005", "VL-CT-005", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-DBL-001", "VL-DBL-001", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-DBL-005", "VL-DBL-005", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-DBL-006", "VL-DBL-006", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-DBL-007", "VL-DBL-007", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-DBL-008", "VL-DBL-008", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-001", "VL-MOT-001", RuleCategory::VoiceLeading));
    rules.push(vl_mot_002_soft());
    rules.push(stub_soft("VL-MOT-003", "VL-MOT-003", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-004", "VL-MOT-004", RuleCategory::VoiceLeading));
    rules.push(vl_mot_005_soft());
    rules.push(stub_soft("VL-MOT-006", "VL-MOT-006", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-007", "VL-MOT-007", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-008", "VL-MOT-008", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-009", "VL-MOT-009", RuleCategory::VoiceLeading));
    rules.push(vl_mot_011_soft());
    rules.push(stub_soft("VL-MOT-013", "VL-MOT-013", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-014", "VL-MOT-014", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-MOT-015", "VL-MOT-015", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-RNG-004", "VL-RNG-004", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-RNG-005", "VL-RNG-005", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-RNG-006", "VL-RNG-006", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-RNG-007", "VL-RNG-007", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-RNG-008", "VL-RNG-008", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-X-001", "VL-X-001", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-X-002", "VL-X-002", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-X-003", "VL-X-003", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-X-005", "VL-X-005", RuleCategory::VoiceLeading));
    rules.push(stub_soft("VL-X-006", "VL-X-006", RuleCategory::VoiceLeading));
    rules
}
