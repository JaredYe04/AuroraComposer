# Performance Targets and Optimization Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Engineering Research Agent (Performance)  
**Dependencies:** [backend.md](../01-architecture/backend.md), [scoring.md](../05-rule-engine/scoring.md), [ADR-003](../../decisions/ADR-003-search-algorithm-primary.md), [ast.md](../02-music-model/ast.md), [roadmap.md](roadmap.md), `research/engineering-research-notes.md`

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

**Appendices:** [A. Benchmark Suite](#appendix-a-benchmark-suite) · [B. Hardware Profiles](#appendix-b-hardware-profiles) · [C. Optimization Checklist](#appendix-c-optimization-checklist)

---

## 1. Background

### 1.1 Purpose

This specification defines **performance targets**, **optimization strategies**, and **measurement methodology** for Aurora Composer. The primary user-facing SLA:

> **Generate a 32-bar, 4-voice composition in <30 seconds on a 4-core desktop CPU.**

Secondary targets cover preview latency, export speed, and UI responsiveness. Performance optimizations must not violate explainability (Principle 2) or bypass hard constraints (Principle 3).

### 1.2 Reference Workload

**Standard benchmark composition:**

| Property | Value |
|----------|-------|
| Length | 32 bars |
| Time signature | 4/4 |
| Voices | 4 (melody, alto, tenor, bass) |
| Style | Classical/pop hybrid preset |
| Search stages | Harmony (beam 8), Melody (beam 16), Counterpoint (beam 12), Bass (beam 8) |
| Rules | Full 395-rule catalog active |
| CPU | 4 cores / 8 threads, 3.0+ GHz (2020-era desktop) |
| Memory budget | ≤512 MB peak for engine |

### 1.3 Scope

**In scope:** Parallel beam search, AST copy-on-write, caching layers, profiling, benchmark CI.

**Out of scope:** GPU acceleration, cloud scaling, ML inference performance (Phase 3 plugins).

---

## 2. Existing Solutions

### 2.1 Comparable Systems

| System | Performance Model | Aurora Lesson |
|--------|------------------|---------------|
| **OpenMusic/Strasheela** | CP backtracking; minutes for complex scores | Aurora uses bounded beam for predictable latency |
| **AIVA** (documented) | Cloud GPU; opaque | Aurora targets local CPU |
| **Helio** | Real-time pattern playback | Borrow incremental preview |
| **Music21** | Analysis seconds per chorale | Export validation target |

### 2.2 Rust Performance Patterns

| Pattern | Crate | Aurora Use |
|---------|-------|------------|
| Work-stealing parallelism | `rayon` | Beam branch scoring |
| Persistent data structures | `im`, `rpds` | AST CoW |
| Arena allocation | `bumpalo` | Candidate generation scratch |
| Profile-guided optimization | `cargo pgo` | Phase 3 release builds |
| Benchmarking | `criterion` | Regression detection |

---

## 3. Academic / Theoretical Foundation

### 3.1 Search Complexity

Beam search per stage:

```text
T_stage ≈ (bars × beats_per_bar × beam_width × branch_factor × C_eval) / P

Where:
  C_eval = cost of rule evaluation on one candidate
  P      = effective parallel threads
  branch_factor = average surviving candidates after hard prune
```

For 32 bars × 4 beats × beam 16 × branch ~8 × C_eval ~50µs → ~819 ms serial eval per stage before overhead. Four search-heavy stages → ~3.3s eval budget; remaining time for candidate generation, IR, repair.

### 3.2 Amdahl's Law

Parallel speedup limited by sequential portions:

| Sequential | Fraction |
|------------|----------|
| Pipeline orchestration | ~5% |
| Stage commit to AST | ~10% |
| IR projection + export | ~5% |
| **Parallelizable search** | ~80% |

Maximum speedup on 4 cores ≈ 1 / (0.2 + 0.8/4) = **2.5×** — design must reduce sequential fraction via caching.

### 3.3 Copy-on-Write Amortized Analysis

CoW branching: O(1) per beam branch vs O(n) full tree clone. For beam 16 × 128 steps × 4 stages:

- Full clone (500 KB AST): ~1 GB copied → unacceptable
- CoW (Arc + overlay): ~16 × 128 × 4 = 8192 cheap branches → acceptable

Reference: Okasaki, *Purely Functional Data Structures* — amortized O(1) cons.

---

## 4. Engineering Analysis

| Criterion | Target |
|-----------|--------|
| **32-bar 4-voice generation** | <30s (p95) |
| **2-bar preview** | <3s (p95) |
| **Parameter tweak debounce preview** | <3s after debounce |
| **MusicXML export (32 bar)** | <500ms |
| **IR projection** | <100ms |
| **UI frame time** | <16ms (60 fps piano roll) |
| **Peak memory** | <512 MB engine |

### 4.1 Performance Budget Breakdown (Target)

| Phase | Budget (ms) | % |
|-------|-------------|---|
| Stages 1–4 (structure, theme) | 2000 | 7% |
| Stage 5 Harmony search | 4000 | 13% |
| Stage 7 Melody search | 8000 | 27% |
| Stage 8 Counterpoint search | 7000 | 23% |
| Stage 9 Bass search | 3000 | 10% |
| Stages 10–13 (drums, decor, repair, validation) | 3000 | 10% |
| IR + export (if inline) | 1000 | 3% |
| Overhead + margin | 4000 | 13% |
| **Total** | **30000** | **100%** |

---

## 5. Comparison of Approaches

### 5.1 Parallelism Granularity

| Granularity | Pros | Cons |
|-------------|------|------|
| Parallel per stage | Simple | Stages mostly sequential |
| **Parallel per beam branch (recommended)** | High utilization | Sync at prune step |
| Parallel per voice | Natural split | Cross-voice rules break |
| Parallel per bar | Low coupling | Global form rules break |

### 5.2 Caching Strategy

| Cache | Hit Rate Expected | Invalidation Cost |
|-------|-------------------|-------------------|
| Compiled RuleSet | High | Plugin reload |
| Theme library | Medium | Theme param change |
| Chord templates | High | Key/style change |
| IR projection | Medium | Any AST patch |
| Rule eval memo (local) | Low-Medium | Per search step |

### 5.3 AST CoW Implementation

| Implementation | Pros | Cons |
|----------------|------|------|
| **`Arc<Composition>` + overlay map (recommended)** | Rust-native | Merge on commit |
| Full `im` tree | Pure functional | Heavier deps |
| Clone-on-write `Vec` per node | Simple | Still O(n) deep clone |

---

## 6. Recommended Solution

Three-pillar optimization strategy:

1. **Parallel beam search** — Rayon parallel over branch scoring each expansion step
2. **AST copy-on-write** — Arc-based snapshots; overlay patches during search
3. **Multi-tier caching** — RuleSet, theme library, chord templates, IR revision cache

Combined with **adaptive beam width** (preview vs full) and **incremental pipeline** (reuse stages 1–4 on param tweak).

---

## 7. Architecture

### 7.1 Performance-Critical Path

```text
User Generate
    → JobManager (spawn_blocking)
        → PipelineOrchestrator
            → [Cache lookup: partial pipeline?]
            → For each search stage:
                  BeamSearch::expand()
                    → par_iter branches
                    → RuleEngine::eval (memoized predicates)
                    → prune + rank
                  commit winner (materialize CoW)
            → IRProjector (cached by revision)
            → Export (optional)
```

### 7.2 Parallel Beam Search Architecture

```rust
// aurora-rules/src/search/beam.rs
pub struct BeamSearch<'a> {
    pool: &'a ThreadPool,
    rule_engine: &'a RuleEngine,
    config: BeamConfig,
}

impl BeamSearch<'_> {
    pub fn expand_step(&self, beam: &[SearchState]) -> Vec<SearchState> {
        beam.par_iter()
            .flat_map(|state| self.generate_candidates(state))
            .filter(|c| self.hard_prune(c))
            .map(|c| self.score_soft(c))
            .collect::<Vec<_>>()
            // sequential: sort + truncate to beam_width
    }
}
```

**Sync points:** Hard prune and top-K selection run sequentially (O(beam log beam)); parallel only candidate eval.

### 7.3 EngineCaches

```rust
pub struct EngineCaches {
    pub ruleset: LruCache<RulesetKey, Arc<CompiledRuleSet>>,
    pub themes: LruCache<ThemeKey, Arc<ThemeLibrary>>,
    pub chords: LruCache<ChordKey, Arc<ChordTemplateSet>>,
    pub ir: LruCache<IrKey, Arc<MusicIr>>,
}
```

Initialized in `src-tauri` at startup; passed via `EngineContext`.

### 7.4 Profiling Integration

| Tool | Use |
|------|-----|
| `tracing` + `tracing-flame` | Stage-level spans |
| `criterion` | Benchmark regression |
| `dhat` (optional) | Heap profiling |
| Chrome trace export | Dev investigation |

Span names: `pipeline.stage`, `search.expand`, `rules.eval`, `ast.cow_branch`, `export.musicxml`.

---

## 8. Data Structures

### 8.1 AstSnapshot (CoW)

```rust
pub struct AstSnapshot {
    root: Arc<Composition>,
    overlay: PatchOverlay,  // HashMap<NodeId, NodePatch>
    generation: u64,
}

impl AstSnapshot {
    pub fn branch(&self) -> Self {
        Self {
            root: Arc::clone(&self.root),
            overlay: PatchOverlay::new(),
            generation: self.generation + 1,
        }
    }

    pub fn get_node(&self, id: NodeId) -> Cow<'_, AstNode> {
        if let Some(patch) = self.overlay.get(id) {
            Cow::Owned(patch.apply(self.root.get_node(id)))
        } else {
            Cow::Borrowed(self.root.get_node(id))
        }
    }
}
```

### 8.2 Cache Keys

```rust
#[derive(Hash, Eq, PartialEq)]
pub struct RulesetKey {
    pub catalog_version: u32,
    pub param_fingerprint: u64,  // hash of weight-relevant params
}

#[derive(Hash, Eq, PartialEq)]
pub struct ThemeKey {
    pub motif_hash: u64,
    pub theme_count: u8,
    pub repetition_ratio_bits: u32,
}

#[derive(Hash, Eq, PartialEq)]
pub struct ChordKey {
    pub key: KeySignature,
    pub genre: String,
    pub complexity_bucket: u8,  // quantized 0-10
}
```

### 8.3 SearchState (Performance-Oriented)

```rust
pub struct SearchState {
    pub snapshot: AstSnapshot,
    pub step_index: u32,
    pub accumulated_score: f64,
    pub beam_rank: u16,
    pub parent_id: Option<StateId>,
    // Inline cache for hot fields — avoid AST walk
    pub last_pitch: [Option<u8>; MAX_VOICES],
    pub current_measure: u16,
}
```

---

## 9. Algorithms

### 9.1 Parallel Beam Search (Detailed)

```text
function parallel_beam_search(initial, config, pool):
    beam = [initial]
    for step in 0..config.max_steps:
        candidates = pool.parallel_flat_map(beam, expand_and_score)
        candidates = hard_prune_all(candidates)      // sequential
        candidates = sort_by_score_desc(candidates)   // sequential
        beam = candidates[0..config.beam_width]
        if cancel_token.is_cancelled(): abort
        report_progress(step / max_steps)
    return beam[0]
```

**Expand-and-score** is parallel; sort/prune sequential.

**Micro-optimization:** Pre-allocate candidate vector capacity `beam_width × branch_factor`.

### 9.2 Rule Evaluation Memoization

Within one expand step, identical predicate contexts may repeat:

```rust
struct EvalCache {
    hits: HashMap<(RuleId, ContextHash), RuleResult>,
}

// Cleared each expand step — not across steps (context changes)
```

Effective for range checks, key membership — ~20% eval reduction in profiling estimates.

### 9.3 Theme Library Cache

```text
on ThemePlanning stage:
  key = hash(motif_params, style)
  if caches.themes.get(key):
    return cached library
  else:
    lib = generate_theme_library(params)
    caches.themes.insert(key, lib)
    return lib
```

Theme generation cost ~200ms uncached → <1ms cached hit.

### 9.4 Chord Template Cache

Precomputed progression templates per `(key, genre, complexity)`:

```text
templates = [
  ["I", "IV", "V", "I"],
  ["I", "vi", "IV", "V"],
  ["ii", "V", "I"],
  ...
]
```

Harmony skeleton selects and transposes — avoids search when `harmony.template_mode = true`.

### 9.5 Incremental Pipeline Reuse

When user changes only `melody.*` params:

```text
Reuse cached AST after stage 6 (rhythm skeleton)
Re-run stages 7–13 only
Estimated savings: ~40% wall time
```

Checkpoint hash: `hash(params.stage_affecting_mask, ast_revision)`.

### 9.6 Adaptive Beam Width

| Mode | Melody Beam | Counterpoint Beam | Target Time |
|------|-------------|-------------------|-------------|
| Preview (2 bar) | 4 | 4 | <3s |
| Standard | 16 | 12 | <30s |
| High quality | 32 | 24 | <90s |

`search.quality_preset` maps to beam configs.

---

## 10. Interfaces

### 10.1 Benchmark API

```rust
// aurora-engine/benches/full_generation.rs
fn bench_32bar_4voice(c: &mut Criterion) {
    let params = BenchmarkParams::standard_32bar_4voice();
    c.bench_function("generate_32bar_4voice", |b| {
        b.iter(|| {
            orchestrator.run_full(black_box(params.clone()), &ctx).unwrap()
        })
    });
}
```

### 10.2 Performance Telemetry

```rust
pub struct PerfReport {
    pub total_ms: u64,
    pub stage_ms: HashMap<StageId, u64>,
    pub search expansions: u64,
    pub cache_hits: CacheHitReport,
    pub peak_rss_bytes: u64,
}
```

Returned in debug builds via `generate_composition` with `debug.perf = true`.

### 10.3 Cache Admin Commands (Tauri)

| Command | Purpose |
|---------|---------|
| `perf_get_report` | Last job PerfReport |
| `cache_clear` | Dev: invalidate all caches |
| `cache_stats` | Hit rates |

---

## 11. Parameter Mappings

| User Parameter | Performance Effect |
|----------------|-------------------|
| `search.beam_width` | Linear on search eval time |
| `search.threads` | Parallelism up to core count |
| `search.quality_preset` | Maps to beam bundle |
| `form.length_bars` | Linear on all stages |
| `engine.cache_enabled` | Toggle EngineCaches |
| `preview.mode` | 2-bar + reduced beam |
| `harmony.template_mode` | Skip harmony search → template lookup |
| `counterpoint.strictness` | Higher → more prune → fewer candidates (faster but harder) |

**Non-linear interaction:** High strictness + low beam → search exhaustion risk ([backend.md](../01-architecture/backend.md) error handling).

---

## 12. Explainability Model

Performance optimizations **must preserve**:

| Optimization | Explainability Impact | Mitigation |
|--------------|----------------------|------------|
| Parallel eval | None — same scores | Deterministic tie-break |
| CoW snapshots | None — same committed AST | Provenance on commit |
| Rule memo | None — same rule results | Cache key includes full context |
| Template harmony | Reduced search provenance | Mark `source = template` |
| Cache reuse | Same output | Log cache hit in debug |

**Deterministic mode:** `search.seed` fixed + stable sort → reproducible results for tests.

---

## 13. Future Expansion

| Enhancement | Expected Gain |
|-------------|---------------|
| SIMD pitch class operations | 10–15% rule eval |
| PGO release builds | 5–10% overall |
| Persistent disk cache (themes) | Cross-session |
| WASM SIMD for browser demo | Phase 3 |
| GPU batch scoring | Not planned v0.1 |
| Distributed beam (multi-machine) | Research only |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| P1 | Optimal LRU size for theme cache? |
| P2 | Cross-stage parallel (harmony + rhythm)? |
| P3 | Accept `harmony.template_mode` default for speed? |
| P4 | Criterion regression threshold (%)? |
| P5 | Memory cap enforcement vs swap? |

---

## 15. References

- [Backend Architecture](../01-architecture/backend.md)
- [Scoring and Search](../05-rule-engine/scoring.md)
- [ADR-003: Beam Search](../../decisions/ADR-003-search-algorithm-primary.md)
- [Music AST — CoW](../02-music-model/ast.md)
- [Testing Strategy](testing.md)
- [Development Roadmap](roadmap.md)
- Okasaki, *Purely Functional Data Structures*
- Blumofe & Leiserson, work-stealing scheduling

---

## Appendix A: Benchmark Suite

| Benchmark ID | Workload | Target (p95) |
|--------------|----------|--------------|
| `BENCH-001` | 32-bar 4-voice full pipeline | <30s |
| `BENCH-002` | 2-bar preview | <3s |
| `BENCH-003` | Melody stage only 32-bar | <8s |
| `BENCH-004` | Counterpoint 32-bar | <7s |
| `BENCH-005` | Harmony beam 32-bar | <4s |
| `BENCH-006` | AST CoW 10k branches | <100ms |
| `BENCH-007` | Rule eval 1M candidates | <2s |
| `BENCH-008` | MusicXML export 32-bar | <500ms |
| `BENCH-009` | IR projection 32-bar | <100ms |
| `BENCH-010` | Incremental re-gen (melody param only) | <18s |

Run: `cargo bench -p aurora-engine`

---

## Appendix B: Hardware Profiles

| Profile | CPU | RAM | Expected BENCH-001 |
|---------|-----|-----|-------------------|
| **Minimum** | 4c/4t 2.5 GHz | 8 GB | <45s (below SLA — warn user) |
| **Target** | 4c/8t 3.0 GHz | 16 GB | <30s |
| **Recommended** | 8c/16t 3.5 GHz | 32 GB | <15s |
| **Developer** | Apple M2 / Ryzen 5800X | 32 GB | <12s |

SLA validated on **Target** profile only.

---

## Appendix C: Optimization Checklist

Phase 2 implementation order:

- [ ] Implement `AstSnapshot` CoW with criterion BENCH-006
- [ ] Rayon pool in JobManager (not global pool)
- [ ] Parallel `expand_step` in BeamSearch
- [ ] EngineCaches: RuleSet + chord templates
- [ ] Eval memoization per expand step
- [ ] Pipeline checkpoint reuse (incremental)
- [ ] Preview mode beam reduction
- [ ] `tracing` spans for all stages
- [ ] BENCH-001 in nightly CI with regression alert >10%

---

*End of Performance Targets and Optimization Specification v0.1*
