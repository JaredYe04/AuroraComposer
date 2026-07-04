//! Jazz rules from `docs/03-theory/jazz.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(jazz_001_hard());
    rules.push(stub_hard("JAZZ-004", "JAZZ-004", RuleCategory::Jazz));
    rules.push(stub_hard("JAZZ-SUB-002", "JAZZ-SUB-002", RuleCategory::Jazz));
    rules.push(jazz_voice_001_hard());
    rules.push(stub_hard("JAZZ-VOICE-006", "JAZZ-VOICE-006", RuleCategory::Jazz));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(stub_soft("JAZZ-002", "JAZZ-002", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-003", "JAZZ-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-005", "JAZZ-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-006", "JAZZ-006", RuleCategory::Jazz));
    rules.push(jazz_007_soft());
    rules.push(stub_soft("JAZZ-008", "JAZZ-008", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-001", "JAZZ-EXT-001", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-002", "JAZZ-EXT-002", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-003", "JAZZ-EXT-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-004", "JAZZ-EXT-004", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-005", "JAZZ-EXT-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-006", "JAZZ-EXT-006", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-007", "JAZZ-EXT-007", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-008", "JAZZ-EXT-008", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-009", "JAZZ-EXT-009", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-EXT-010", "JAZZ-EXT-010", RuleCategory::Jazz));
    rules.push(jazz_iiv_001_soft());
    rules.push(stub_soft("JAZZ-IIV-002", "JAZZ-IIV-002", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-003", "JAZZ-IIV-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-004", "JAZZ-IIV-004", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-005", "JAZZ-IIV-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-006", "JAZZ-IIV-006", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-007", "JAZZ-IIV-007", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-008", "JAZZ-IIV-008", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-009", "JAZZ-IIV-009", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-010", "JAZZ-IIV-010", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-011", "JAZZ-IIV-011", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-012", "JAZZ-IIV-012", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-IIV-013", "JAZZ-IIV-013", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-MOD-001", "JAZZ-MOD-001", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-MOD-002", "JAZZ-MOD-002", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-MOD-003", "JAZZ-MOD-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-MOD-004", "JAZZ-MOD-004", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-MOD-005", "JAZZ-MOD-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-001", "JAZZ-SUB-001", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-003", "JAZZ-SUB-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-004", "JAZZ-SUB-004", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-005", "JAZZ-SUB-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-006", "JAZZ-SUB-006", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-007", "JAZZ-SUB-007", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-SUB-008", "JAZZ-SUB-008", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-002", "JAZZ-VOICE-002", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-003", "JAZZ-VOICE-003", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-004", "JAZZ-VOICE-004", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-005", "JAZZ-VOICE-005", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-007", "JAZZ-VOICE-007", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-008", "JAZZ-VOICE-008", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-009", "JAZZ-VOICE-009", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-010", "JAZZ-VOICE-010", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-011", "JAZZ-VOICE-011", RuleCategory::Jazz));
    rules.push(stub_soft("JAZZ-VOICE-012", "JAZZ-VOICE-012", RuleCategory::Jazz));
    rules
}
