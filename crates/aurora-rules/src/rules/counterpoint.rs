//! Counterpoint rules from `docs/03-theory/counterpoint.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(cp_030_hard());
    rules.push(stub_hard("CP-038", "CP-038", RuleCategory::Counterpoint));
    rules.push(stub_hard("CP-056", "CP-056", RuleCategory::Counterpoint));
    rules.push(cp_058_hard());
    rules.push(cp_par_001_hard());
    rules.push(cp_par_002_hard());
    rules.push(stub_hard("CP-PAR-003", "CP-PAR-003", RuleCategory::Counterpoint));
    rules.push(stub_hard("CP-PAR-004", "CP-PAR-004", RuleCategory::Counterpoint));
    rules.push(stub_hard("CP-S4-001", "CP-S4-001", RuleCategory::Counterpoint));
    rules.push(stub_hard("CP-S4-002", "CP-S4-002", RuleCategory::Counterpoint));
    rules.push(stub_hard("CP-S4-003", "CP-S4-003", RuleCategory::Counterpoint));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(cp_par_001_soft_soft());
    rules.push(stub_soft("CP-020", "CP-020", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-021", "CP-021", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-022", "CP-022", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-023", "CP-023", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-024", "CP-024", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-025", "CP-025", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-026", "CP-026", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-027", "CP-027", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-028", "CP-028", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-031", "CP-031", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-032", "CP-032", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-033", "CP-033", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-034", "CP-034", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-035", "CP-035", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-036", "CP-036", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-037", "CP-037", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-050", "CP-050", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-051", "CP-051", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-052", "CP-052", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-053", "CP-053", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-054", "CP-054", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-055", "CP-055", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-057", "CP-057", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-005", "CP-PAR-005", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-006", "CP-PAR-006", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-007", "CP-PAR-007", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-008", "CP-PAR-008", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-009", "CP-PAR-009", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-PAR-010", "CP-PAR-010", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-001", "CP-S1-001", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-002", "CP-S1-002", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-003", "CP-S1-003", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-004", "CP-S1-004", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-005", "CP-S1-005", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-006", "CP-S1-006", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-007", "CP-S1-007", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S1-008", "CP-S1-008", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S2-001", "CP-S2-001", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S2-002", "CP-S2-002", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S2-003", "CP-S2-003", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S2-004", "CP-S2-004", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S2-005", "CP-S2-005", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-001", "CP-S3-001", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-002", "CP-S3-002", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-003", "CP-S3-003", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-004", "CP-S3-004", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-005", "CP-S3-005", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S3-006", "CP-S3-006", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S4-004", "CP-S4-004", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S4-005", "CP-S4-005", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S5-001", "CP-S5-001", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S5-002", "CP-S5-002", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S5-003", "CP-S5-003", RuleCategory::Counterpoint));
    rules.push(stub_soft("CP-S5-004", "CP-S5-004", RuleCategory::Counterpoint));
    rules
}
