//! Theory rule catalog — modular rule sets from `docs/03-theory/*`.
//!
//! ~395 rules registered; 30+ with full evaluators, remainder neutral stubs for explainability.

mod core;
mod counterpoint;
mod form;
mod harmony;
mod helpers;
mod implemented;
mod jazz;
mod orchestration;
mod rhythm;
mod voice_leading;

use crate::rule::RuleSet;

/// Full catalog RuleSet aggregating all theory categories.
#[must_use]
pub fn full_rule_set() -> RuleSet {
    aggregate()
}

/// Prototype / default RuleSet (alias for full catalog in Phase 3).
#[must_use]
pub fn prototype_rule_set() -> RuleSet {
    full_rule_set()
}

fn aggregate() -> RuleSet {
    let mut hard = Vec::new();
    let mut soft = Vec::new();

    for rules in [
        core::hard_rules(),
        harmony::hard_rules(),
        counterpoint::hard_rules(),
        voice_leading::hard_rules(),
        rhythm::hard_rules(),
        form::hard_rules(),
        jazz::hard_rules(),
        orchestration::hard_rules(),
    ] {
        hard.extend(rules);
    }

    for rules in [
        core::soft_rules(),
        harmony::soft_rules(),
        counterpoint::soft_rules(),
        voice_leading::soft_rules(),
        rhythm::soft_rules(),
        form::soft_rules(),
        jazz::soft_rules(),
        orchestration::soft_rules(),
    ] {
        soft.extend(rules);
    }

    RuleSet::new().with_hard(hard).with_soft(soft)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_near_four_hundred_rules() {
        let set = full_rule_set();
        assert!(
            set.rule_count() >= 380,
            "expected ~395 rules, got {}",
            set.rule_count()
        );
    }

    #[test]
    fn rule_ids_are_unique() {
        let set = full_rule_set();
        let mut ids = HashSet::new();
        for r in &set.hard {
            assert!(ids.insert(r.meta.id.as_str().to_string()), "duplicate {}", r.meta.id);
        }
        for r in &set.soft {
            assert!(ids.insert(r.meta.id.as_str().to_string()), "duplicate {}", r.meta.id);
        }
    }

    #[test]
    fn critical_rules_are_registered() {
        let set = full_rule_set();
        let ids: HashSet<_> = set
            .hard
            .iter()
            .map(|r| r.meta.id.as_str())
            .chain(set.soft.iter().map(|r| r.meta.id.as_str()))
            .collect();
        for id in [
            "HARM-001",
            "CP-PAR-001",
            "VL-MOT-002",
            "RHY-SYNC-001",
            "FORM-PHR-002",
            "JAZZ-007",
            "ORCH-010",
            "REG-001",
        ] {
            assert!(ids.contains(id), "missing {id}");
        }
    }
}
