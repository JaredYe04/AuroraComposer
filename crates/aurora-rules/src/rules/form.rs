//! Form rules from `docs/03-theory/form.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(stub_hard("FORM-ABA-008", "FORM-ABA-008", RuleCategory::Form));
    rules.push(stub_hard("FORM-SEC-008", "FORM-SEC-008", RuleCategory::Form));
    rules.push(stub_hard("FORM-SON-001", "FORM-SON-001", RuleCategory::Form));
    rules.push(stub_hard("FORM-SON-002", "FORM-SON-002", RuleCategory::Form));
    rules.push(stub_hard("FORM-SON-004", "FORM-SON-004", RuleCategory::Form));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(stub_soft("FORM-001", "FORM-001", RuleCategory::Form));
    rules.push(stub_soft("FORM-002", "FORM-002", RuleCategory::Form));
    rules.push(stub_soft("FORM-003", "FORM-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-004", "FORM-004", RuleCategory::Form));
    rules.push(stub_soft("FORM-005", "FORM-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-006", "FORM-006", RuleCategory::Form));
    rules.push(stub_soft("FORM-007", "FORM-007", RuleCategory::Form));
    rules.push(stub_soft("FORM-008", "FORM-008", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-001", "FORM-ABA-001", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-002", "FORM-ABA-002", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-003", "FORM-ABA-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-004", "FORM-ABA-004", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-005", "FORM-ABA-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-006", "FORM-ABA-006", RuleCategory::Form));
    rules.push(stub_soft("FORM-ABA-007", "FORM-ABA-007", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-001", "FORM-DEV-001", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-002", "FORM-DEV-002", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-003", "FORM-DEV-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-004", "FORM-DEV-004", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-005", "FORM-DEV-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-006", "FORM-DEV-006", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-007", "FORM-DEV-007", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-008", "FORM-DEV-008", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-009", "FORM-DEV-009", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-010", "FORM-DEV-010", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-011", "FORM-DEV-011", RuleCategory::Form));
    rules.push(stub_soft("FORM-DEV-012", "FORM-DEV-012", RuleCategory::Form));
    rules.push(form_phr_001_soft());
    rules.push(form_phr_002_soft());
    rules.push(stub_soft("FORM-PHR-003", "FORM-PHR-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-PHR-004", "FORM-PHR-004", RuleCategory::Form));
    rules.push(stub_soft("FORM-PHR-005", "FORM-PHR-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-001", "FORM-SEC-001", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-002", "FORM-SEC-002", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-003", "FORM-SEC-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-004", "FORM-SEC-004", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-005", "FORM-SEC-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-006", "FORM-SEC-006", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-007", "FORM-SEC-007", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-009", "FORM-SEC-009", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-010", "FORM-SEC-010", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-011", "FORM-SEC-011", RuleCategory::Form));
    rules.push(stub_soft("FORM-SEC-012", "FORM-SEC-012", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-003", "FORM-SON-003", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-005", "FORM-SON-005", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-006", "FORM-SON-006", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-007", "FORM-SON-007", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-008", "FORM-SON-008", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-009", "FORM-SON-009", RuleCategory::Form));
    rules.push(stub_soft("FORM-SON-010", "FORM-SON-010", RuleCategory::Form));
    rules
}
