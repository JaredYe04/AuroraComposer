//! Harmony rules from `docs/03-theory/harmony.md`.

use super::helpers::{stub_hard, stub_soft};
use super::implemented::*;
use crate::rule::{HardRule, RuleCategory, SoftRule};

pub fn hard_rules() -> Vec<HardRule> {
    let mut rules = Vec::new();
    rules.push(stub_hard("HARM-002", "HARM-002", RuleCategory::Harmony));
    rules.push(harm_008_hard());
    rules.push(harm_026_hard());
    rules.push(stub_hard("HARM-027", "HARM-027", RuleCategory::Harmony));
    rules.push(harm_041_hard());
    rules.push(harm_mel_002_hard());
    rules.push(harm_mel_003_hard());
    rules.push(harm_050_hard());
    rules.push(stub_hard("HARM-066", "HARM-066", RuleCategory::Harmony));
    rules.push(harm_cad_007_hard());
    rules.push(stub_hard("HARM-CAD-010", "HARM-CAD-010", RuleCategory::Harmony));
    rules.push(stub_hard("HARM-PROG-015", "HARM-PROG-015", RuleCategory::Harmony));
    rules
}

pub fn soft_rules() -> Vec<SoftRule> {
    let mut rules = Vec::new();
    rules.push(harm_001_soft());
    rules.push(harm_mel_001_soft());
    rules.push(harm_003_soft());
    rules.push(harm_015_soft());
    rules.push(stub_soft("HARM-004", "HARM-004", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-005", "HARM-005", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-006", "HARM-006", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-007", "HARM-007", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-009", "HARM-009", RuleCategory::Harmony));
    rules.push(harm_010_soft());
    rules.push(stub_soft("HARM-020", "HARM-020", RuleCategory::Harmony));
    rules.push(harm_021_soft());
    rules.push(stub_soft("HARM-022", "HARM-022", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-023", "HARM-023", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-024", "HARM-024", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-025", "HARM-025", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-028", "HARM-028", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-040", "HARM-040", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-042", "HARM-042", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-043", "HARM-043", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-044", "HARM-044", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-045", "HARM-045", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-060", "HARM-060", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-061", "HARM-061", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-062", "HARM-062", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-063", "HARM-063", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-064", "HARM-064", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-065", "HARM-065", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-067", "HARM-067", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-068", "HARM-068", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-001", "HARM-CAD-001", RuleCategory::Harmony));
    rules.push(harm_cad_002_soft());
    rules.push(stub_soft("HARM-CAD-003", "HARM-CAD-003", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-004", "HARM-CAD-004", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-005", "HARM-CAD-005", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-006", "HARM-CAD-006", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-008", "HARM-CAD-008", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-CAD-009", "HARM-CAD-009", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-001", "HARM-PROG-001", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-002", "HARM-PROG-002", RuleCategory::Harmony));
    rules.push(harm_prog_003_soft());
    rules.push(stub_soft("HARM-PROG-004", "HARM-PROG-004", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-005", "HARM-PROG-005", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-006", "HARM-PROG-006", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-007", "HARM-PROG-007", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-008", "HARM-PROG-008", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-009", "HARM-PROG-009", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-010", "HARM-PROG-010", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-011", "HARM-PROG-011", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-012", "HARM-PROG-012", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-013", "HARM-PROG-013", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-PROG-014", "HARM-PROG-014", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-001", "HARM-VOICE-001", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-002", "HARM-VOICE-002", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-003", "HARM-VOICE-003", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-004", "HARM-VOICE-004", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-005", "HARM-VOICE-005", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-006", "HARM-VOICE-006", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-007", "HARM-VOICE-007", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-008", "HARM-VOICE-008", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-009", "HARM-VOICE-009", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-010", "HARM-VOICE-010", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-011", "HARM-VOICE-011", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-012", "HARM-VOICE-012", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-013", "HARM-VOICE-013", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-014", "HARM-VOICE-014", RuleCategory::Harmony));
    rules.push(stub_soft("HARM-VOICE-015", "HARM-VOICE-015", RuleCategory::Harmony));
    rules
}
