# Coding Style and Conventions Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Engineering Research Agent (Documentation / Style)  
**Dependencies:** [backend.md](../01-architecture/backend.md), [frontend.md](../01-architecture/frontend.md), [terminology.md](../00-overview/terminology.md), [acas-v0.1.md](../00-overview/acas-v0.1.md), `research/engineering-research-notes.md`

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

**Appendices:** [A. Rust Style Guide](#appendix-a-rust-style-guide) · [B. TypeScript / Vue Style Guide](#appendix-b-typescript--vue-style-guide) · [C. Cross-Language Naming](#appendix-c-cross-language-naming) · [D. Tool Configuration](#appendix-d-tool-configuration)

---

## 1. Background

### 1.1 Purpose

This document defines **coding conventions** for Aurora Composer Phase 2 implementation across Rust backend crates and Vue 3 TypeScript frontend. Consistent style reduces review friction, enables automated linting, and aligns code with architectural boundaries defined in [backend.md](../01-architecture/backend.md) and [frontend.md](../01-architecture/frontend.md).

### 1.2 Principles

| Principle | Style Implication |
|-----------|-------------------|
| Specification before code | Public APIs match spec interfaces first |
| Everything Is Music AST | Domain types use glossary terms (`Voice`, not `Track`) |
| Everything Is Explainable | Provenance fields never optional on generated events |
| Modular crates | `pub` surface minimal; internals module-private |

### 1.3 Scope

**In scope:** Naming, formatting, lint rules, documentation comments, error handling patterns, test naming, commit message guidance.

**Out of scope:** Git branching strategy, code review assignment (see team process docs).

---

## 2. Existing Solutions

### 2.1 Rust Style References

| Standard | Application |
|----------|-------------|
| [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/) | Public API design |
| `rustfmt` defaults | Formatting — no custom unless justified |
| `clippy::pedantic` | Lint with project allowlist |
| Rust RFC 1574 (error patterns) | `thiserror` in libraries |

### 2.2 TypeScript / Vue References

| Standard | Application |
|----------|-------------|
| [Vue 3 Style Guide](https://vuejs.org/style-guide/) — Priority A & B | Component structure |
| [TypeScript handbook](https://www.typescriptlang.org/docs/handbook/) | Strict mode |
| ESLint `recommended` + Vue plugin | Lint |
| Prettier | Formatting |

### 2.3 Cross-Language IPC

Serde JSON field names: **`snake_case`** on wire (matches Rust). TypeScript types mirror with same casing for 1:1 mapping.

---

## 3. Academic / Theoretical Foundation

Coding conventions implement **cognitive load reduction** (Miller, chunking) and **consistent metaphor** (Lakoff) — musicians and developers share domain vocabulary from [terminology.md](../00-overview/terminology.md).

**Clean Architecture** (Martin): dependency direction inward toward domain (`aurora-ast`) — reflected in crate lint rules and import restrictions.

---

## 4. Engineering Analysis

| Criterion | Approach |
|-----------|----------|
| Consistency | Automated formatters mandatory pre-commit |
| Readability | Max function length ~60 lines; extract when exceeded |
| Safety | `#![deny(unsafe_code)]` in all crates except explicit FFI module |
| Testability | One assertion concept per test |
| Documentation | `///` on all public items; link to spec sections |

---

## 5. Comparison of Approaches

### 5.1 Rust Error Handling

| Pattern | Use |
|---------|-----|
| `thiserror` enums | Library crates (`aurora-*`) |
| `anyhow` | `src-tauri` binary only — not in library public API |
| Panic | Bug only; never user input path |

### 5.2 Vue Component Style

| Pattern | Use |
|---------|-----|
| `<script setup lang="ts">` | All components |
| Options API | **Forbidden** for new code |
| Composables | Shared logic extraction |
| Global mixins | **Forbidden** |

### 5.3 Async Patterns

| Layer | Pattern |
|-------|---------|
| Tauri commands | `async fn` wrapping `spawn_blocking` |
| Vue | `async/await` in stores; no raw Promise chains |
| Rust engine | Sync — parallelism via Rayon only |

---

## 6. Recommended Solution

Adopt **rustfmt + clippy + thiserror** for Rust and **ESLint + Prettier + strict TypeScript** for Vue, with **shared glossary naming** across IPC boundary.

Pre-commit hooks (Phase 2):

```text
cargo fmt --check
cargo clippy -- -D warnings
npm run lint
npm run format:check
```

---

## 7. Architecture

### 7.1 Code Organization by Crate

| Crate | Module Pattern |
|-------|---------------|
| `aurora-core` | Flat modules: `error`, `params`, `ids` |
| `aurora-ast` | Domain modules: `composition`, `event`, `patch`, `snapshot` |
| `aurora-rules` | `dsl/`, `engine`, `constraint`, `search/` |
| `aurora-engine` | `stages/<stage_name>.rs`, `orchestrator.rs` |
| `aurora-export` | `ir`, `musicxml/`, `midi/`, `abc/` |
| `src-tauri` | `commands/`, `jobs/`, `project/` |
| `frontend` | Feature folders under `components/` |

### 7.2 Visibility Rules

```rust
// lib.rs — re-export minimal public surface
pub use error::AuroraError;
pub use composition::{Composition, Event, Voice};

// Internal modules
mod patch;  // pub(crate) types only
```

No `pub use` re-export chains deeper than one level.

### 7.3 Frontend Component File Structure

```text
components/piano-roll/
├── PianoRoll.vue          # template + script setup
├── PianoRollCanvas.ts     # canvas logic (testable)
├── usePianoRoll.ts        # composable
├── piano-roll.types.ts    # local types
└── PianoRoll.spec.ts      # Vitest
```

One component per `.vue` file; canvas logic extracted for unit testing.

---

## 8. Data Structures

### 8.1 Naming Conventions Summary

| Entity | Rust | TypeScript |
|--------|------|------------|
| Types / Structs | `PascalCase` | `PascalCase` |
| Functions / methods | `snake_case` | `camelCase` |
| Constants | `SCREAMING_SNAKE` | `SCREAMING_SNAKE` |
| Modules / files (Rust) | `snake_case.rs` | — |
| Files (Vue/TS) | — | `PascalCase.vue`, `camelCase.ts` |
| IPC JSON fields | `snake_case` | `snake_case` in DTO types |

### 8.2 Domain Type Names (Binding)

From glossary — **do not alternate synonyms**:

| Correct | Avoid |
|---------|-------|
| `Voice` | `Track`, `Part` (except MusicXML interop) |
| `Composition` | `Score` (except Music21 interop) |
| `Event` | `NoteEvent` (unless disambiguating from `Event` trait) |
| `Measure` | `Bar` (UI labels may say "bar") |
| `Provenance` | `Metadata`, `History` |
| `ParameterBundle` | `Config`, `Settings` (UI settings are app-level) |

### 8.3 Newtype IDs

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuleId(u32);
```

Never use raw `u64` / `String` for IDs in public APIs.

---

## 9. Algorithms

### 9.1 Documentation Comment Template (Rust)

```rust
/// Evaluates all soft rules against `snapshot` and returns accumulated score.
///
/// Hard constraints are **not** evaluated here — use [`ConstraintSolver::prune`].
///
/// # Errors
/// Returns [`AuroraError::Internal`] if rule catalog failed to load.
///
/// See [Scoring spec](../docs/05-rule-engine/scoring.md) §9.
pub fn eval_soft(&self, snapshot: &AstSnapshot) -> Result<f64, AuroraError>
```

### 9.2 Test Naming

```rust
#[test]
fn harmony_stage_writes_chord_events_to_measure() { }

#[test]
fn rule_harm_001_positive_fixture_satisfied() { }

#[test]
fn export_roundtrip_g1_preserves_pitch_rhythm() { }
```

Pattern: `{subject}_{condition}_{expected_outcome}`

### 9.3 Vue Composable Template

```typescript
/**
 * Subscribes to Tauri job progress events and updates job store.
 * @see docs/01-architecture/frontend.md §7.5
 */
export function useJobProgress() {
  const jobStore = useJobStore();
  // ...
  return { progress, status };
}
```

---

## 10. Interfaces

### 10.1 Public Rust API Stability

Phase 2: all `aurora-*` crates version `0.1.x` — breaking changes allowed until Design Freeze.

Mark unstable APIs:

```rust
#[doc(hidden)]
pub fn experimental_feature() { }
```

### 10.2 Serde Conventions

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct GenerationParams {
    pub form_length_bars: u16,
    #[serde(default)]
    pub search_beam_width: Option<u32>,
}
```

Use `#[serde(default)]` for optional params; never omit required fields without default.

### 10.3 Tauri Command Naming

Commands: **`snake_case`** verb phrases:

```rust
#[tauri::command]
async fn generate_composition(params: GenerationParams) -> Result<JobId, AuroraError>

#[tauri::command]
async fn get_timeline_projection(composition_id: String) -> Result<TimelineDto, AuroraError>
```

Events: **`namespace://action`** pattern: `job://progress`, `job://complete`.

---

## 11. Parameter Mappings

Code identifiers for parameters use **dot-path to snake_case**:

| ACAS Path | Rust field | TS field |
|-----------|------------|----------|
| `form.length_bars` | `form_length_bars` | `form_length_bars` |
| `harmony.complexity` | `harmony_complexity` | `harmony_complexity` |
| `search.beam_width` | `search_beam_width` | `search_beam_width` |

Nested structs preferred when parameter groups exceed 5 fields:

```rust
pub struct HarmonyParams {
    pub complexity: f32,
    pub extension_bias: f32,
}
```

---

## 12. Explainability Model

Code conventions supporting explainability:

| Rule | Rationale |
|------|-----------|
| Never `skip` provenance in constructors | Principle 2 |
| Rule IDs as `RuleId` newtype, not strings | Stable Inspector references |
| Log `rule_id` in violation errors | UI display |
| Search provenance fields mandatory on commit | Beam trace |

```rust
// Required on generated events
pub struct Provenance {
    pub stage: StageId,
    pub rule_ids: Vec<RuleId>,
    pub search_step: Option<u32>,
    pub beam_rank: Option<u16>,
    pub score_delta: f64,
}
```

---

## 13. Future Expansion

| Item | Phase |
|------|-------|
| `tauri-specta` type export | 3 |
| Custom `rustfmt` max width 100 | If team prefers |
| Architecture decision lint (crate deps) | 3 — `cargo-deny` |
| i18n string externalization | 3 |
| Plugin SDK style guide extension | 3 |

---

## 14. Open Questions

| ID | Question |
|----|----------|
| S1 | Allow `anyhow` in integration tests? |
| S2 | Max line width 100 vs 120? |
| S3 | Mandatory Signed-off-by commits? |
| S4 | English-only comments — enforced? |

---

## 15. References

- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Vue Style Guide](https://vuejs.org/style-guide/)
- [Terminology Glossary](../00-overview/terminology.md)
- [Backend Architecture](../01-architecture/backend.md)
- [Frontend Architecture](../01-architecture/frontend.md)
- [Testing Strategy](testing.md)

---

## Appendix A: Rust Style Guide

### A.1 Formatting

- Run `cargo fmt` — default rustfmt profile
- Edition 2021
- Max line length: 100 (rustfmt `max_width = 100`)

### A.2 Clippy Allowlist

`clippy.toml` / crate attributes:

```rust
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]  // aurora_ast::ast acceptable
#![allow(clippy::must_use_candidate)]
```

Deny in CI:

- `clippy::unwrap_used` — use `?` or explicit handling in production code
- `clippy::expect_used` — allowed in tests only
- `clippy::panic` — deny in library crates

### A.3 Import Order

```rust
// 1. std
use std::collections::HashMap;

// 2. external crates
use rayon::prelude::*;
use serde::{Deserialize, Serialize};

// 3. aurora crates
use aurora_ast::{Composition, Event};
use aurora_core::AuroraError;

// 4. crate-local
use crate::search::BeamSearch;
```

### A.4 Error Handling

```rust
// Library — typed errors
pub fn eval(&self, ast: &Composition) -> Result<Score, AuroraError> { }

// Don't
pub fn eval(&self, ast: &Composition) -> Score {  // no panic on user data
    self.inner.eval(ast).unwrap()
}
```

### A.5 Unsafe Code

```rust
// Only in aurora-export/src/ffi/ or plugin boundary
// Requires SAFETY comment block per function
```

---

## Appendix B: TypeScript / Vue Style Guide

### B.1 TypeScript Config

```json
{
  "compilerOptions": {
    "strict": true,
    "noUncheckedIndexedAccess": true,
    "moduleResolution": "bundler",
    "paths": { "@/*": ["src/*"] }
  }
}
```

### B.2 Component Rules

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import type { TimelineDto } from '@/types/composition';

const props = defineProps<{
  timeline: TimelineDto;
}>();

const emit = defineEmits<{
  'select-measure': [measure: number];
}>();
</script>
```

- Use `defineProps<T>()` type syntax
- Prefer `ref` + `computed` over `reactive` for primitives
- No default export besides component

### B.3 ESLint Rules (Key)

- `@typescript-eslint/no-explicit-any`: error
- `@typescript-eslint/explicit-function-return-type`: warn on exports
- `vue/component-name-in-template-casing`: PascalCase
- `vue/multi-word-component-names`: error (except `App.vue`)

### B.4 Prettier

```json
{
  "semi": true,
  "singleQuote": true,
  "printWidth": 100,
  "trailingComma": "es5"
}
```

---

## Appendix C: Cross-Language Naming

| Concept | Rust | TypeScript | JSON IPC |
|---------|------|------------|----------|
| Job identifier | `JobId` | `JobId` | `"job_id": "uuid"` |
| Composition | `Composition` | `CompositionHandle` | `composition_id` |
| Beam width param | `search_beam_width` | `search_beam_width` | `search_beam_width` |
| Stage progress | `StageProgress` | `StageProgress` | `stage_name`, `percent` |

---

## Appendix D: Tool Configuration

### D.1 Repository Root Files (Phase 2)

```text
rustfmt.toml
clippy.toml (optional)
Cargo.toml          # workspace.lints
frontend/
  .eslintrc.cjs
  .prettierrc
  tsconfig.json
.pre-commit-config.yaml
```

### D.2 Commit Message Format

```
<type>(<scope>): <subject>

<body optional>
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

Scopes: `ast`, `rules`, `engine`, `export`, `tauri`, `ui`, `spec`

Example: `feat(engine): implement melody beam search stage`

### D.3 PR Requirements

- [ ] `cargo test --workspace` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `npm run lint && npm run test` passes
- [ ] Spec reference in PR description for API changes
- [ ] No `#![allow(unused)]` without issue link

---

*End of Coding Style and Conventions Specification v0.1*
