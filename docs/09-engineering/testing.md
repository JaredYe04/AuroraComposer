# Testing Strategy Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Engineering Research Agent (Testing)  
**Dependencies:** [backend.md](../01-architecture/backend.md), [frontend.md](../01-architecture/frontend.md), [rule-dsl.md](../05-rule-engine/rule-dsl.md), [musicxml.md](../06-export/musicxml.md), [acas-v0.1.md](../00-overview/acas-v0.1.md), `research/theory-research-notes.md`, `research/engineering-research-notes.md`

---

## Table of Contents

1. [Background](#1-background)
2. [Existing Solutions](#2-existing-solutions)
3. [Academic / Theoretical Foundation](#3-academic--theoretical-foundation)
4. [Engineering Analysis](#4-engineering-analysis)
5. [Comparison of Approaches](#5-comparison-of-approaches)
6. [Recommended Solution](#6-recommended-solution)
7. [Architecture](#7-architecture)
8. [Data Structures](#8-data-structures)
9. [Algorithms](#9-algorithms)
10. [Interfaces](#10-interfaces)
11. [Parameter Mappings](#11-parameter-mappings)
12. [Explainability Model](#12-explainability-model)
13. [Future Expansion](#13-future-expansion)
14. [Open Questions](#14-open-questions)
15. [References](#15-references)

**Appendices:** [A. Test Category Matrix](#appendix-a-test-category-matrix) · [B. Fixture Layout](#appendix-b-fixture-layout) · [C. Subjective Listening Protocol](#appendix-c-subjective-listening-protocol) · [D. Music21 Validation Harness](#appendix-d-music21-validation-harness)

---

## 1. Background

### 1.1 Purpose

This specification defines Aurora Composer's **testing strategy** across Rust backend crates, Vue frontend, and cross-system validation. Testing verifies:

- Module correctness (unit tests per crate)
- Theory rule fidelity (395 rules from `docs/03-theory/`)
- Parameter responsiveness (output changes with input)
- Export round-trip guarantees (MusicXML G1–G5)
- Subjective musical quality (listening test protocol)
- External analysis validation (Music21-style checks)

Testing is **mandatory before Design Freeze exit criteria** and gates Phase 2 prototype acceptance per [roadmap.md](roadmap.md).

### 1.2 Quality Goals

From [goals.md](../00-overview/goals.md) and [deep-research-report.md](../../deep-research-report.md):

| Goal | Test Category |
|------|---------------|
| Explainability | Provenance present on all generated events |
| Parameterization | Monotonic parameter response tests |
| Theory correctness | Rule validation + Music21 analysis |
| Export fidelity | G1–G5 round-trip tests |
| Performance | Benchmarks in [performance.md](performance.md) |
| User satisfaction | Subjective listening protocol |

### 1.3 Scope

**In scope:** Test pyramid, fixtures, CI integration, rule corpus tests, export golden files, listening study design.

**Out of scope:** Production load testing, security penetration testing (Phase 3).

---

## 2. Existing Solutions

### 2.1 Music21 Testing Patterns

Music21 uses:

- **Corpus tests** — run analyzer on known scores
- **Unit tests** per module (`test_stream`, `test_interval`)
- **`freezeExpected`/`compare** for serialized output

Aurora adopts **corpus-style validation** for Bach chorales and **golden files** for MusicXML export.

### 2.2 Rust Testing Conventions

| Type | Location | Tooling |
|------|----------|---------|
| Unit | `#[cfg(test)]` in module | `cargo test` |
| Integration | `crates/*/tests/` | `cargo test --test` |
| Benchmark | `benches/` | `criterion` |
| Doc tests | `///` examples | `cargo test --doc` |

### 2.3 Vue / Tauri Testing

| Layer | Tool |
|-------|------|
| Vue unit | Vitest + `@vue/test-utils` |
| Composable | Vitest |
| E2E | Playwright + Tauri WebDriver (Phase 3) |
| IPC contract | Rust integration tests invoke command handlers directly |

---

## 3. Academic / Theoretical Foundation

### 3.1 Objective vs Subjective Evaluation

Music generation quality requires **dual validation**:

| Type | Method | Ground Truth |
|------|--------|--------------|
| **Objective** | Rule pass rate, Music21 interval analysis, export schema | Theory textbooks, Bach corpus |
| **Subjective** | Listening tests (MUSHRA-adapted) | Human perception |

Reference: Collins, *Algorithmic Composition* (2018) — evaluation frameworks for rule-based systems.

### 3.2 Rule Validation Methodology

Each of **395 theory rules** (catalog in `research/theory-research-notes.md`) requires:

1. **Positive fixture** — AST fragment that satisfies rule → score bonus / no violation
2. **Negative fixture** — AST fragment that violates → penalty / hard prune
3. **Edge case** — style-dependent boundary (e.g., parallel fifths in jazz vs species CP)

Rule prefix distribution:

| Domain | Prefix | Count |
|--------|--------|-------|
| Harmony | HARM | 78 |
| Counterpoint | CP | 62 |
| Voice leading | VL | 47 |
| Rhythm | RHY | 54 |
| Form | FORM | 58 |
| Jazz | JAZZ | 56 |
| Orchestration | ORCH | 40 |
| **Total** | | **395** |

### 3.3 Export Fidelity Theory

MusicXML round-trip guarantees G1–G5 ([musicxml.md](../06-export/musicxml.md) Appendix D.3):

| Guarantee | Definition |
|-----------|------------|
| **G1** | Pitch/rhythm Tier A elements survive Aurora ↔ MusicXML ↔ Aurora |
| **G2** | Provenance Tier B metadata survives Aurora round-trip |
| **G3** | Tier A survives export → MuseScore 4 → import |
| **G4** | Verovio renders without error (`preview` profile) |
| **G5** | No silent data loss — unmappable nodes logged |

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| **Correctness** | 395 rule tests + integration pipeline tests |
| **Controllability** | Parameter grid tests with statistical assertions |
| **Explainability** | Provenance completeness tests |
| **Performance** | Criterion benchmarks (separate doc) |
| **Extensibility** | Plugin test harness stub |
| **Complexity** | Large fixture corpus — automated generation helpers needed |

### 4.1 CI Pipeline Stages

```text
1. cargo fmt --check && cargo clippy
2. cargo test --workspace
3. cargo test --test rule_corpus
4. cargo test --test export_roundtrip
5. npm run test (Vitest)
6. criterion --baseline (nightly only)
7. music21_validate (optional Python job)
```

---

## 5. Comparison of Approaches

### 5.1 Rule Test Authoring

| Approach | Pros | Cons |
|----------|------|------|
| Manual Rust test per rule | Precise | 395× boilerplate |
| **Data-driven `.rule` + fixture YAML (recommended)** | Scalable | Parser dependency |
| Property-based (Proptest) | Finds edge cases | Hard for music semantics |

### 5.2 Music21 Integration

| Approach | Pros | Cons |
|----------|------|------|
| **Python subprocess (recommended Phase 2)** | Full Music21 | CI Python dep |
| Rust port of analyzers | No Python | High effort |
| Skip external validation | Faster CI | Weaker confidence |

### 5.3 Golden File Management

| Approach | Pros | Cons |
|----------|------|------|
| Commit XML to repo | Reproducible | Large diffs |
| **Compressed fixtures + hash (recommended)** | Smaller | Update workflow |
| Snapshot on every CI run | Always fresh | Non-deterministic failures |

---

## 6. Recommended Solution

Adopt a **four-layer test pyramid**:

```text
                    ┌─────────────┐
                    │  Subjective │  Listening tests (manual schedule)
                    │  Listening  │
                    ├─────────────┤
                    │  E2E / IPC  │  Tauri command flows
                    ├─────────────┤
                    │ Integration │  Pipeline, export round-trip, Music21
                    ├─────────────┤
                    │    Unit     │  Per-crate, per-rule, per-stage
                    └─────────────┘
```

**Data-driven rule corpus** in `tests/fixtures/rules/` drives 395 rule validation tests.

**Export round-trip** golden suite in `tests/fixtures/musicxml/`.

**Parameter responsiveness** via grid tests in `aurora-engine/tests/param_grid/`.

---

## 7. Architecture

### 7.1 Test Layout by Crate

```text
crates/
├── aurora-core/tests/           # error serialization, param parsing
├── aurora-ast/tests/            # patch, CoW, provenance invariants
├── aurora-rules/tests/
│   ├── rule_corpus/             # 395 rule fixtures
│   └── search/                  # beam, A*, DP correctness
├── aurora-engine/tests/
│   ├── pipeline/                # stage integration
│   ├── param_grid/              # responsiveness
│   └── bach_chorale/            # end-to-end style validation
├── aurora-export/tests/
│   ├── musicxml_roundtrip/      # G1–G5
│   ├── midi_smoke/
│   └── golden/                  # reference files
src-tauri/tests/                 # IPC integration
frontend/
├── src/**/*.spec.ts             # Vitest unit
└── e2e/                         # Playwright (Phase 3)
tests/
├── integration/                 # cross-crate
├── music21/                     # Python validation scripts
└── listening/                   # protocol docs + result templates
```

### 7.2 Unit Tests Per Module

| Module / Crate | Test Focus | Example Tests |
|----------------|------------|---------------|
| `aurora-core` | Serde round-trip, error codes | `test_job_id_serialize` |
| `aurora-ast` | Patch apply, CoW branch, invariants | `test_cow_branch_independent` |
| `aurora-rules` | DSL compile, single rule eval | `test_harm_001_major_triad` |
| `aurora-engine` | Stage I/O, orchestrator order | `test_melody_stage_writes_voice` |
| `aurora-export` | IR projection, XML validity | `test_divisions_consistent` |
| `src-tauri` | Command handler validation | `test_generate_invalid_params` |
| Frontend stores | Pinia actions mock IPC | `jobStore.generate` |

### 7.3 Rule Validation Test Architecture

```text
tests/fixtures/rules/
├── HARM/
│   ├── HARM-001-positive.yaml
│   ├── HARM-001-negative.yaml
│   └── ...
├── CP/
├── VL/
├── RHY/
├── FORM/
├── JAZZ/
└── ORCH/

RuleCorpusTest harness:
  for each rule_id in catalog:
    load positive fixture → assert satisfied (or score > 0)
    load negative fixture → assert violated (or pruned)
    if rule.has_param_binding:
      run weight sweep test
```

```rust
// aurora-rules/tests/rule_corpus.rs
#[test]
fn rule_corpus_all_rules() {
    let catalog = RuleCatalog::load("tests/fixtures/rules/catalog.json");
    for rule in catalog.rules() {
        run_rule_fixture(&rule, FixtureKind::Positive);
        run_rule_fixture(&rule, FixtureKind::Negative);
    }
}
```

CI fails if catalog count ≠ 395 or any rule lacks both fixtures.

---

## 8. Data Structures

### 8.1 Rule Fixture Schema (YAML)

```yaml
# tests/fixtures/rules/HARM/HARM-001-positive.yaml
rule_id: HARM-001
description: "Tonic triad in major key is stable"
kind: positive
ast: !include ../snippets/c_major_triad_measure.ast.json
params:
  harmony.complexity: 0.3
expect:
  satisfied: true
  min_score: 10.0
```

```yaml
# negative variant
rule_id: HARM-001
kind: negative
ast: !include ../snippets/diminished_on_tonic.ast.json
expect:
  satisfied: false
  hard_prune: false
  min_penalty: 5.0
```

### 8.2 Parameter Grid Test Case

```json
{
  "id": "param-harmony-complexity-monotonic",
  "base_params": { "form.length_bars": 8, "style.genre": "classical" },
  "sweep": {
    "param": "harmony.complexity",
    "values": [0.1, 0.3, 0.5, 0.7, 0.9]
  },
  "metric": "extended_chord_ratio",
  "assertion": "monotonic_increasing",
  "tolerance": 0.05
}
```

### 8.3 Export Round-Trip Test Case

```json
{
  "id": "rt-g1-g2-32bar-4voice",
  "source": "fixtures/compositions/pop_32bar_4voice.ast.json",
  "profile": "interchange",
  "guarantees": ["G1", "G2", "G5"],
  "steps": [
    "export_musicxml",
    "import_musicxml",
    "assert_ast_equivalent_tier_a",
    "assert_provenance_preserved"
  ]
}
```

### 8.4 Listening Test Session Record

```yaml
session_id: LISTEN-2026-Q3-01
date: 2026-09-15
protocol_version: "1.0"
panels:
  - style: pop_happy
    samples: [gen_001, gen_002, gen_003]
    reference: ref_pop_happy.mid
raters: 12
dimensions: [style_match, emotion_match, overall_quality, would_use]
```

---

## 9. Algorithms

### 9.1 AST Equivalence Check (Tier A)

```rust
fn assert_ast_equivalent_tier_a(original: &Composition, roundtrip: &Composition) {
    assert_eq!(original.voice_count(), roundtrip.voice_count());
    for (e1, e2) in original.events().zip(roundtrip.events()) {
        assert_eq!(e1.pitch_midi(), e2.pitch_midi());
        assert_eq!(e1.duration_ticks(), e2.duration_ticks());
        assert_eq!(e1.beat_offset(), e2.beat_offset());
        // spelling may differ if enharmonic — compare MIDI
    }
}
```

### 9.2 Parameter Responsiveness Metric Extraction

After generation with swept parameter:

```rust
fn extended_chord_ratio(ast: &Composition) -> f64 {
    let chords = ast.harmony_events();
    let extended = chords.filter(|c| c.has_seventh_or_above()).count();
    extended as f64 / chords.count().max(1) as f64
}
```

Assert monotonicity or threshold per test case JSON.

### 9.3 Music21 Validation Pipeline

```python
# tests/music21/validate_score.py
def validate(path: str) -> ValidationReport:
    score = corpus.parse(path)
    report = ValidationReport()
    report.parallel_fifths = detect_parallel_fifths(score)
    report.parallel_octaves = detect_parallel_octaves(score)
    report.key_clarity = estimate_key_clarity(score)
    report.range_violations = check_ranges(score, preset="choral")
    return report
```

Invoked from Rust integration test via `std::process::Command`.

### 9.4 Statistical Test for Parameter Grid

For monotonic assertion with tolerance:

```text
for i in 1..n:
  assert metric[i] >= metric[i-1] - tolerance
```

For categorical params (genre), assert discrete expectation sets (e.g., jazz → ii-V-I frequency > threshold).

---

## 10. Interfaces

### 10.1 Test Harness CLI (Phase 2)

```bash
# Run full rule corpus
cargo test -p aurora-rules --test rule_corpus

# Run export guarantees
cargo test -p aurora-export --test musicxml_roundtrip -- --nocapture

# Parameter grid subset
cargo test -p aurora-engine --test param_grid -- harmony

# Music21 validation on generated MIDI
python tests/music21/validate_score.py target/generated.mid --preset bach
```

### 10.2 CI Configuration (GitHub Actions sketch)

```yaml
jobs:
  rust-test:
    steps:
      - run: cargo test --workspace --all-features
      - run: cargo test -p aurora-rules --test rule_corpus
  frontend-test:
    steps:
      - run: npm ci && npm run test
  music21:
    if: github.event_name == 'schedule'
    steps:
      - run: pip install music21 && python tests/music21/run_batch.py
```

### 10.3 Coverage Targets (Phase 2)

| Crate | Line Coverage Target |
|-------|---------------------|
| `aurora-core` | 90% |
| `aurora-ast` | 85% |
| `aurora-rules` | 80% (+ 100% rule fixture coverage) |
| `aurora-engine` | 70% (algorithms grow Phase 3) |
| `aurora-export` | 85% |

---

## 11. Parameter Mappings

Test configuration parameters:

| Test Param | Effect |
|------------|--------|
| `TEST_SEED` | Fixed RNG seed for reproducible generation |
| `TEST_BEAM_WIDTH` | Override to 4 in unit tests for speed |
| `TEST_BAR_COUNT` | Default 4 bars in unit tests; 32 in integration |
| `MUSIC21_ENABLED` | Skip Python validation when false |
| `GOLDEN_UPDATE` | Regenerate golden XML files (local only) |

Musical parameters under test — see Parameter Grid catalog in `tests/fixtures/param_grid/`.

Key responsiveness tests:

| Parameter | Expected Metric Change |
|-----------|----------------------|
| `harmony.complexity` ↑ | Extended chord ratio ↑ |
| `counterpoint.strictness` ↑ | Parallel fifth count ↓ |
| `melody.ornament_density` ↑ | Ornament event count ↑ |
| `drums.density` ↑ | Drum hits per bar ↑ |
| `search.beam_width` ↑ | Score variance ↓ (quality stabilizes) |
| `emotion.valence` ↓ | Minor mode ratio ↑ |

---

## 12. Explainability Model

Tests verify explainability requirements:

| Test | Assertion |
|------|-----------|
| `test_all_generated_events_have_provenance` | Every `Event::Note` has non-empty `Provenance` |
| `test_beam_provenance_fields` | `beam_rank`, `search_step` present when search stage |
| `test_manual_edit_provenance` | UI patch sets `source = manual_edit` |
| `test_export_provenance_g2` | Round-trip preserves Tier B metadata |
| `test_inspector_dto_complete` | IPC provenance query returns rule IDs + reasons |

Failure of any explainability test blocks release.

---

## 13. Future Expansion

| Phase | Testing Enhancement |
|-------|---------------------|
| 2 | Core unit + rule corpus + G1/G2 round-trip |
| 3 | Full 395 rules; G3 MuseScore automation; E2E Playwright |
| 3 | Fuzz testing AST patches (Proptest) |
| 3 | Continuous listening panel (quarterly) |
| 3 | Performance regression in CI (criterion compare) |
| 3 | Plugin certification test suite |

---

## 14. Open Questions

| ID | Question | Impact |
|----|----------|--------|
| T1 | Auto-generate negative fixtures from rule DSL? | Rule corpus effort |
| T2 | MuseScore CLI headless in CI Docker? | G3 automation |
| T3 | Minimum listening panel size (10 vs 20)? | Statistical power |
| T4 | Accept Music21 subprocess in required CI? | Dev environment |
| T5 | Snapshot test piano roll rendering? | Frontend |

---

## 15. References

- [Backend Architecture](../01-architecture/backend.md)
- [MusicXML Specification §G1–G5](../06-export/musicxml.md)
- [Rule DSL](../05-rule-engine/rule-dsl.md)
- [Theory Research Notes — 395 rules](../../research/theory-research-notes.md)
- [Development Roadmap](roadmap.md)
- Music21 documentation — analysis tools
- ITU-R BS.1534 (MUSHRA) — subjective audio test reference
- Collins, *Algorithmic Composition* (2018)

---

## Appendix A: Test Category Matrix

| Category | Scope | Count Target | Automation |
|----------|-------|--------------|------------|
| **Unit — core** | `aurora-core` | 50+ | Full |
| **Unit — ast** | `aurora-ast` | 100+ | Full |
| **Unit — rules** | `aurora-rules` | 395×2 fixtures | Full |
| **Unit — engine stages** | per stage | 20+ each | Full |
| **Unit — export** | serializers | 80+ | Full |
| **Integration — pipeline** | end-to-end stages | 30+ | Full |
| **Parameter responsiveness** | grid cases | 40+ | Full |
| **Export round-trip G1–G5** | golden compositions | 25+ | Full |
| **Music21 validation** | corpus scores | 50+ | Scheduled |
| **Subjective listening** | style panels | 4 panels × 3 samples | Manual |
| **Performance benchmarks** | criterion | 15+ | Nightly |
| **Frontend unit** | Vitest | 60+ | Full |
| **E2E IPC** | Tauri | 10+ | Phase 3 |

---

## Appendix B: Fixture Layout

```text
tests/fixtures/
├── rules/
│   ├── catalog.json              # 395 rule index
│   ├── HARM/ CP/ VL/ RHY/ FORM/ JAZZ/ ORCH/
│   └── snippets/                 # shared AST fragments
├── compositions/
│   ├── pop_32bar_4voice.ast.json
│   ├── bach_chorale_style.ast.json
│   ├── jazz_16bar.ast.json
│   └── edge_cases/
├── param_grid/
│   ├── harmony_complexity.json
│   ├── counterpoint_strictness.json
│   └── ...
├── musicxml/
│   ├── golden/
│   │   ├── pop_32bar.interchange.xml
│   │   └── ...
│   └── external/
│       └── museScore_samples/
└── midi/
    └── listening_refs/
```

---

## Appendix C: Subjective Listening Test Protocol

### C.1 Purpose

Validate that parameter changes produce **perceptually consistent** style and emotion outcomes — not captured by rule pass rates alone.

### C.2 Panel

| Requirement | Value |
|-------------|-------|
| Minimum raters | 12 |
| Preferred background | Mixed: 4 musicians, 4 hobbyists, 4 general |
| Environment | Quiet headphones, calibrated volume |
| Duration | ≤45 minutes per session |

### C.3 Stimuli

Per panel, generate **3 algorithmic samples** + **1 reference** (human-composed or curated MIDI):

| Panel ID | Params | Reference |
|----------|--------|-----------|
| POP-HAPPY | C major, high valence, medium complexity | Pop backing track |
| CLASS-SAD | A minor, low valence, high complexity | Slow classical excerpt |
| JAZZ-MEDIUM | Bb, jazz genre, medium swing | Jazz standard lead sheet |
| ELECTRONIC-DENSE | E minor, high drum density | Electronic loop |

Samples generated with fixed `TEST_SEED` for reproducibility; different seeds across the 3 variants.

### C.4 Rating Scales

| Dimension | Scale | Question |
|-----------|-------|----------|
| Style match | 0–100 | "How well does this match the labeled style?" |
| Emotion match | 0–100 | "How well does this convey the intended emotion?" |
| Overall quality | 0–100 | "How musically satisfying is this excerpt?" |
| Would use | Yes/No | "Would you use this in a project?" |

Optional MUSHRA-style hidden reference for Phase 3.

### C.5 Acceptance Criteria

| Metric | Threshold |
|--------|-----------|
| Style match mean | ≥ 60 per panel |
| Emotion match mean | ≥ 55 per panel |
| Algorithm vs reference gap | ≤ 25 points on overall quality |
| Parameter A/B discrimination | Significant (p < 0.05) on key metric when param swept |

### C.6 Procedure

1. Brief rater training (2 reference examples)
2. Randomized sample order (blind — no provenance shown)
3. 30-second excerpt per sample (middle 8 bars of 32-bar generation)
4. Rest break at midpoint
5. Collect forms → aggregate in `tests/listening/results/`

---

## Appendix D: Music21 Validation Harness

### D.1 Checks

| Check | Music21 Method | Pass Criteria |
|-------|---------------|---------------|
| Parallel fifths | Custom voice pair scan | 0 for strict counterpoint preset |
| Parallel octaves | Custom voice pair scan | 0 for strict preset |
| Key clarity | `analyze('key')` | Matches requested key ≥80% measures |
| Range | Part range vs preset | 0 violations |
| Cadence presence | Harmonic analysis | Authentic cadence every 8 bars (classical) |

### D.2 Bach Chorale Regression

371 Bach chorales as **upper bound reference** — Aurora strict classical preset should not exceed violation rate of:

- Parallel fifths: 0 (species strict)
- Hard rule violations: 0

Generated chorale-style 8-bar excerpts compared statistically (not note-for-note match).

### D.3 Integration

```rust
#[test]
#[ignore = "requires music21"]
fn bach_style_music21_clean() {
    let ast = generate_fixture("bach_chorale_8bar");
    let midi = export_midi(&ast);
    let report = music21::validate(&midi, "bach_strict");
    assert_eq!(report.parallel_fifths, 0);
    assert_eq!(report.parallel_octaves, 0);
}
```

---

*End of Testing Strategy Specification v0.1*
