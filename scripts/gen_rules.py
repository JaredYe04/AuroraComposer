#!/usr/bin/env python3
"""Generate aurora-rules category modules from extracted ID lists."""
import pathlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
RULES_DIR = ROOT / "crates/aurora-rules/src/rules"

CATEGORY = {
    "harmony": "RuleCategory::Harmony",
    "counterpoint": "RuleCategory::Counterpoint",
    "voice_leading": "RuleCategory::VoiceLeading",
    "rhythm": "RuleCategory::Rhythm",
    "form": "RuleCategory::Form",
    "jazz": "RuleCategory::Jazz",
    "orchestration": "RuleCategory::Orchestration",
}

DOC_NAME = {"voice_leading": "voice-leading"}

HARD_IMPL = {
    "harmony": {
        "HARM-008", "HARM-026", "HARM-041", "HARM-050", "HARM-CAD-007",
    },
    "counterpoint": {"CP-PAR-001", "CP-PAR-002", "CP-030", "CP-058"},
    "voice_leading": {"VL-MOT-010", "VL-MOT-011", "VL-RNG-001", "VL-RNG-002", "VL-DBL-002"},
    "rhythm": {"RHY-SUB-001", "RHY-MTR-001"},
    "form": set(),
    "jazz": {"JAZZ-001", "JAZZ-VOICE-001"},
    "orchestration": {"ORCH-001"},
}

SOFT_IMPL = {
    "harmony": {
        "HARM-001", "HARM-003", "HARM-010", "HARM-015", "HARM-021",
        "HARM-PROG-003", "HARM-CAD-002",
    },
    "counterpoint": {"CP-PAR-001-soft"},
    "voice_leading": {"VL-CT-001", "VL-MOT-002", "VL-MOT-005"},
    "rhythm": {"RHY-MTR-003", "RHY-SYNC-001", "RHY-GRO-003"},
    "form": {"FORM-PHR-001", "FORM-PHR-002"},
    "jazz": {"JAZZ-007", "JAZZ-IIV-001"},
    "orchestration": {"ORCH-002", "ORCH-010"},
}

HARD_STUB = {
    "HARM-002", "HARM-027", "HARM-066", "HARM-CAD-010", "HARM-PROG-015",
    "CP-PAR-003", "CP-PAR-004", "CP-S4-001", "CP-S4-002", "CP-S4-003", "CP-038", "CP-056",
    "VL-DBL-003", "VL-DBL-004", "VL-X-004", "VL-RNG-003",
    "RHY-SUB-002", "RHY-SUB-007", "RHY-GRO-001", "RHY-GRO-014",
    "FORM-SEC-008", "FORM-SON-001", "FORM-SON-002", "FORM-SON-004", "FORM-ABA-008",
    "JAZZ-004", "JAZZ-SUB-002", "JAZZ-VOICE-006",
    "ORCH-003", "ORCH-015",
}


def fn_name(rid: str) -> str:
    return rid.lower().replace("-", "_")


def generate_category(name: str) -> None:
    doc = DOC_NAME.get(name, name)
    ids = (RULES_DIR / f"{name}_ids.txt").read_text(encoding="utf-8").splitlines()
    cat = CATEGORY[name]
    hard_impl = HARD_IMPL.get(name, set())
    soft_impl = SOFT_IMPL.get(name, set())

    lines = [
        f"//! {name.replace('_', ' ').title()} rules from `docs/03-theory/{doc}.md`.",
        "",
        "use super::helpers::{stub_hard, stub_soft};",
        "use super::implemented::*;",
        "use crate::rule::{HardRule, RuleCategory, SoftRule};",
        "",
        "pub fn hard_rules() -> Vec<HardRule> {",
        "    let mut rules = Vec::new();",
    ]
    for rid in ids:
        if rid in hard_impl:
            lines.append(f"    rules.push({fn_name(rid)}_hard());")
        elif rid in HARD_STUB:
            lines.append(f'    rules.push(stub_hard("{rid}", "{rid}", {cat}));')
    lines += [
        "    rules",
        "}",
        "",
        "pub fn soft_rules() -> Vec<SoftRule> {",
        "    let mut rules = Vec::new();",
    ]
    for rid in ids:
        if rid in soft_impl:
            lines.append(f"    rules.push({fn_name(rid)}_soft());")
        elif rid not in hard_impl and rid not in HARD_STUB:
            lines.append(f'    rules.push(stub_soft("{rid}", "{rid}", {cat}));')
    lines += ["    rules", "}"]
    (RULES_DIR / f"{name}.rs").write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    for n in CATEGORY:
        generate_category(n)
    print("generated", len(CATEGORY), "category modules")
