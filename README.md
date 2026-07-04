# Aurora Composer

**Version:** 0.1  
**Status:** Phase 2 Prototype — Core engine working  
**Design Freeze:** ✅ 2026-07-04

---

## What Is Aurora Composer?

A parameterized, explainable, extensible **Music Composition Engine** — rule-driven search grounded in music theory, not AI.

> **Music Theory + Rule System + Constraint + Search**

---

## Quick Start

```bash
# Run all Rust tests (52 tests)
cargo test --workspace

# Build and run desktop app
cd ui && npm install && npm run build
cd ../src-tauri && cargo run
```

---

## Repository Layout

```text
crates/
  aurora-core/     IDs, errors, ParameterBundle
  aurora-ast/      Music AST, provenance, CBOR projects
  aurora-rules/    Rule engine, beam search (23 prototype rules)
  aurora-engine/   Pipeline: structure → harmony → melody
  aurora-export/   IR projection, MIDI, MusicXML
src-tauri/         Tauri 2 desktop shell
ui/                Vue 3 + TypeScript frontend
docs/              ACAS specifications (source of truth)
decisions/         8 Architecture Decision Records
reviews/           Design Freeze Report
```

---

## Documentation

| Document | Description |
|----------|-------------|
| [ACAS v0.1](docs/00-overview/acas-v0.1.md) | Master specification |
| [Design Freeze](reviews/design-freeze-v0.1.md) | Phase 2 authorization |
| [Progress](PROGRESS.md) | Current status |
| [Docs Index](docs/README.md) | Full specification tree |

---

## Prototype Demo

The UI provides:
- Parameter controls (key, style, beam width, bar count)
- **Generate** — runs full pipeline with progress events
- **Export MIDI / MusicXML** — download generated files

Every generated note carries **provenance metadata** (rule ID, score, reason).

---

## Development Methodology

```
Research → Specification → Review → Freeze → Implementation
```

Phase 1 complete (49 specs, 395 theory rules). Phase 2 prototype core complete.
