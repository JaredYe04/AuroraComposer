# Aurora Composer

**Version:** 0.1  
**Status:** Phase 3 — Production Core Complete  
**Tests:** 66 passing | **Rules:** 401 | **Pipeline:** 14 stages

---

## Quick Start

```bash
cargo test --workspace          # 66 tests
cd ui && npm install && npm run build
cd ../src-tauri && cargo run    # Launch desktop app
```

---

## What It Does

A **parameterized, explainable music composition engine**:

- Generates multi-voice scores (melody, counterpoint, bass, drums)
- Every note has **provenance** (rule ID, score, reason)
- **401 music theory rules** with beam search
- Export to **MIDI, MusicXML, ABC, SVG**
- **Vue 3 UI**: timeline, piano roll, inspector, playback

---

## Architecture

```text
crates/
  aurora-core/      Parameters, errors, config
  aurora-ast/       Music AST + provenance + CBOR projects
  aurora-rules/     401 rules + beam search
  aurora-plugin/    Style plugin host (classical/jazz/pop)
  aurora-engine/    14-stage generation pipeline
  aurora-export/    MIDI, MusicXML, ABC, SVG
src-tauri/          Tauri 2 desktop shell
ui/                 Vue 3 workspace
docs/               ACAS specifications (source of truth)
```

---

## Documentation

- [ACAS v0.1](docs/00-overview/acas-v0.1.md)
- [Design Freeze](reviews/design-freeze-v0.1.md)
- [Progress](PROGRESS.md)
- [Docs Index](docs/README.md)
