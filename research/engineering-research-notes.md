# Engineering Research Notes (Raw)

**Agent:** Engineering Research Agent  
**Date:** 2026-07-04  
**Status:** Raw notes — not normative; see `docs/01-architecture/` and `docs/09-engineering/` for specifications

---

## 1. Technology Stack Survey

### Rust Backend Frameworks

| Option | Pros | Cons | Verdict |
|--------|------|------|---------|
| **Tauri 2.x** | Rust-native IPC, small bundle, capability security | Desktop-first; mobile beta | **Selected** per ACAS |
| **Electron + Rust N-API** | Mature UI ecosystem | Heavy bundle; dual runtime | Rejected |
| **Pure WASM in browser** | No install | Limited file I/O; plugin sandbox harder | Phase 3 optional target |

### Rust Music Libraries (2026)

| Crate | Capability | Aurora Use |
|-------|-----------|------------|
| `rimd` | MIDI read/write | Reference for `aurora-export` MIDI |
| `midly` | Lightweight MIDI | Candidate dependency |
| `quick-xml` | Streaming XML | MusicXML serializer |
| `roxmltree` | DOM parse | MusicXML import |
| `rayon` | Data parallelism | Beam branch evaluation |
| `tokio` | Async runtime | Tauri job manager I/O |
| `thiserror` / `anyhow` | Error ergonomics | `aurora-core` error types |
| `im` / `rpds` | Persistent data structures | AST CoW evaluation |
| `serde` / `serde_json` | IPC serialization | Tauri command payloads |

No mature Rust MusicXML 4.0 crate exists — Aurora implements in `aurora-export`.

### Frontend Frameworks

| Option | Notes |
|--------|-------|
| **Vue 3 + TypeScript** | ACAS + ADR-002; Composition API, Pinia |
| React | Deep research report suggestion; superseded |
| Svelte | Smaller ecosystem for music viz |

Component library candidates: **Naive UI** (Vue 3, TypeScript), **PrimeVue**, custom canvas for piano roll.

### Audio / Notation

| Tool | Role |
|------|------|
| Tone.js | WebAudio MIDI synthesis for preview |
| Verovio WASM | MusicXML → SVG for in-app notation |
| abcjs | ABC preview (Phase 3) |
| FluidSynth | Optional Tauri-side render (complex; deferred) |

---

## 2. Crate Structure Research

### Monorepo Layout (Cargo Workspace)

```
aurora-composer/
├── Cargo.toml              # workspace root
├── crates/
│   ├── aurora-core/        # shared types, errors, config, job IDs
│   ├── aurora-ast/         # Music AST, patches, CoW snapshots
│   ├── aurora-rules/       # Rule DSL compile, evaluate, constraint
│   ├── aurora-engine/      # pipeline, algorithm stages, search
│   └── aurora-export/      # IR projection, MusicXML/MIDI/ABC
├── src-tauri/              # Tauri app: IPC, JobManager, ProjectStore
└── frontend/               # Vue 3 app
```

### Dependency DAG

```text
aurora-core          (no internal deps)
    ↑
aurora-ast           (core)
    ↑
aurora-rules         (ast)
    ↑
aurora-engine        (ast, rules)
    ↑
aurora-export        (ast, core)
    ↑
src-tauri            (all crates)
```

### Alternative Considered: Single `aurora` Crate

Rejected for Phase 2+. Compile times and test isolation suffer; plugin SDK needs stable `aurora-ast` + `aurora-rules` surface without pulling full engine.

---

## 3. Concurrency Model Notes

### Tauri Async vs Rayon Sync

| Workload | Runtime | Rationale |
|----------|---------|-----------|
| IPC command dispatch | Tauri main + async commands | Non-blocking UI |
| Full pipeline job | `tokio::spawn_blocking` → Rayon pool | CPU-bound; avoid blocking async executor |
| Beam branch scoring | Rayon `par_iter` | Embarrassingly parallel |
| Export file I/O | Tokio async fs | Disk latency |
| Progress events | `tauri::Emitter` from worker thread | Cross-thread safe |

### Job Cancellation

- `CancellationToken` (tokio-util) shared into pipeline
- Check at stage boundaries and every N search steps
- Preview jobs: aggressive cancellation on parameter change

### Thread Pool Sizing

```rust
// Default: num_cpus - 1, min 2, max 8
rayon::ThreadPoolBuilder::new()
    .num_threads(config.search_threads)
    .thread_name(|i| format!("aurora-search-{i}"))
    .build()
```

User parameter `search.threads` overrides default (Principle 4).

---

## 4. AST Copy-on-Write Research

### Candidates

| Approach | Pros | Cons |
|----------|------|------|
| **Full clone per branch** | Simple | O(tree size × beam width) — too slow |
| **`Arc` + path copying** | Cheap sharing | Complex patch semantics |
| **`im::Vector` / persistent maps** | Functional updates | Learning curve |
| **Arena + generation counter** | Fast allocation | Harder to serialize |

**Recommendation:** Hybrid — `Arc<Composition>` root with structural sharing; search states hold `AstSnapshot { root: Arc<Composition>, overlay: PatchMap }`. Commit merges overlay into new Arc on stage completion.

Reference: `rpds` RedBlackTreeMap for overlay patches keyed by `NodeId`.

---

## 5. Error Handling Patterns

### Rust Ecosystem Convention

- Library crates: `thiserror` enums per domain
- Tauri boundary: map to `AuroraError` serializable for IPC
- Never expose internal panics to frontend

### Error Categories Identified

1. **User/recoverable** — unsatisfiable constraints, invalid params
2. **System** — I/O, plugin load failure
3. **Internal** — invariant violation (bug); log + generic message to UI

---

## 6. Testing Strategy Survey

### Reference Projects

| Project | Testing Approach | Aurora Adoption |
|---------|-----------------|-----------------|
| **Music21** | Corpus tests, `unittest` | Music21-style analysis validation |
| **Rust std** | Unit + integration + doc tests | Per-crate `tests/` |
| **Tauri** | WebDriver + Rust integration | E2E for IPC commands |
| **Verovio** | Regression SVG diffs | PDF preview smoke tests |

### 395 Rule Validation

Theory catalog totals 395 rules across 7 domains (see `research/theory-research-notes.md`). Each rule needs:

- Positive fixture (satisfies rule)
- Negative fixture (violates rule)
- Parameter weight binding test (optional)

Automated via `aurora-rules` test harness reading `.rule` files + golden AST snippets.

### Performance Benchmarks

Use `criterion` crate:

- `bench_beam_melody_32bar_4voice`
- `bench_ast_cow_branch`
- `bench_musicxml_roundtrip_500measure`

Target: <30s for 32-bar multi-voice on 4-core CPU (roadmap Phase 2 criterion).

---

## 7. Frontend Architecture Notes

### State Management

| Layer | Store | Contents |
|-------|-------|----------|
| Project | Pinia `useProjectStore` | AST handle, metadata, dirty flag |
| UI | Pinia `useUiStore` | selection, panel layout, theme |
| Generation | Pinia `useJobStore` | job ID, progress, cancel token |
| Parameters | Pinia `useParamStore` | bound ACAS parameter tree |

AST never fully deserialized in frontend — Tauri returns projection DTOs for timeline/piano roll.

### IPC Command Inventory (Draft)

| Command | Direction | Payload |
|---------|-----------|---------|
| `generate_composition` | FE → BE | `GenerationParams` |
| `cancel_job` | FE → BE | `JobId` |
| `get_composition_summary` | FE → BE | `ProjectId` |
| `apply_ast_patch` | FE → BE | `AstPatch` |
| `export_musicxml` | FE → BE | `ExportProfile` |
| `job_progress` | BE → FE (event) | `StageProgress` |
| `job_complete` | BE → FE (event) | `CompositionHandle` |

---

## 8. Caching Opportunities

| Cache | Key | Invalidation |
|-------|-----|--------------|
| Theme library | `(motif_hash, params_fingerprint)` | Param change in theme.* |
| Chord templates | `(key, style, complexity)` | Style plugin reload |
| RuleSet compile | `(ruleset_version, param_hash)` | Plugin hot-load |
| IR projection | `(ast_revision)` | Any AST patch |
| MusicXML DOM | `(ir_revision, profile)` | Export only |

LRU in-memory; optional disk cache for theme library in Phase 3.

---

## 9. Coding Style References

- Rust: `rustfmt` default, `clippy` pedantic with project allowlist
- TypeScript: ESLint + `@vue/eslint-config-typescript`, Prettier
- Naming: Rust `snake_case`, TS `camelCase`, shared domain terms from glossary

---

## 10. Open Questions Logged

| # | Question | Owner |
|---|----------|-------|
| E1 | Pin persistent AST overlay vs full immutable tree? | AST Agent |
| E2 | Tokio vs dedicated thread for JobManager? | Backend spec |
| E3 | Vitest vs Playwright for Vue E2E? | Frontend spec |
| E4 | Criterion CI regression thresholds? | Performance spec |
| E5 | Music21 validation via Python subprocess or Rust port? | Testing spec |

---

## 11. Documents Produced

| Output | Path |
|--------|------|
| Backend Architecture | `docs/01-architecture/backend.md` |
| Frontend Architecture | `docs/01-architecture/frontend.md` |
| Testing Strategy | `docs/09-engineering/testing.md` |
| Performance Targets | `docs/09-engineering/performance.md` |
| Coding Style | `docs/09-engineering/coding-style.md` |

---

*End of Engineering Research Notes*
