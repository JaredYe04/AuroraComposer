//! Rhythm rules from `docs/03-theory/rhythm.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(stub_hard("RHY-GRO-001", "RHY-GRO-001", RuleCategory::Rhythm));
    rules.push(stub_hard("RHY-GRO-014", "RHY-GRO-014", RuleCategory::Rhythm));
    rules.push(rhy_mtr_001_hard());
    rules.push(rhy_sub_001_hard());
    rules.push(stub_hard("RHY-SUB-002", "RHY-SUB-002", RuleCategory::Rhythm));
    rules.push(stub_hard("RHY-SUB-007", "RHY-SUB-007", RuleCategory::Rhythm));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(stub_soft("RHY-001", "RHY-001", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-002", "RHY-002", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-003", "RHY-003", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-004", "RHY-004", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-005", "RHY-005", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-006", "RHY-006", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-007", "RHY-007", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-008", "RHY-008", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-002", "RHY-GRO-002", RuleCategory::Rhythm));
    rules.push(rhy_gro_003_soft());
    rules.push(stub_soft("RHY-GRO-004", "RHY-GRO-004", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-005", "RHY-GRO-005", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-006", "RHY-GRO-006", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-007", "RHY-GRO-007", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-008", "RHY-GRO-008", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-009", "RHY-GRO-009", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-010", "RHY-GRO-010", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-011", "RHY-GRO-011", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-012", "RHY-GRO-012", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-013", "RHY-GRO-013", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-015", "RHY-GRO-015", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-016", "RHY-GRO-016", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-017", "RHY-GRO-017", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-018", "RHY-GRO-018", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-019", "RHY-GRO-019", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-GRO-020", "RHY-GRO-020", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-002", "RHY-MTR-002", RuleCategory::Rhythm));
    rules.push(rhy_mtr_003_soft());
    rules.push(stub_soft("RHY-MTR-004", "RHY-MTR-004", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-005", "RHY-MTR-005", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-006", "RHY-MTR-006", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-007", "RHY-MTR-007", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-008", "RHY-MTR-008", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-009", "RHY-MTR-009", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-MTR-010", "RHY-MTR-010", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SUB-003", "RHY-SUB-003", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SUB-004", "RHY-SUB-004", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SUB-005", "RHY-SUB-005", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SUB-006", "RHY-SUB-006", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SUB-008", "RHY-SUB-008", RuleCategory::Rhythm));
    rules.push(rhy_sync_001_soft());
    rules.push(stub_soft("RHY-SYNC-002", "RHY-SYNC-002", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-003", "RHY-SYNC-003", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-004", "RHY-SYNC-004", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-005", "RHY-SYNC-005", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-006", "RHY-SYNC-006", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-007", "RHY-SYNC-007", RuleCategory::Rhythm));
    rules.push(stub_soft("RHY-SYNC-008", "RHY-SYNC-008", RuleCategory::Rhythm));
    rules
}
