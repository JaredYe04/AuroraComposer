# Aurora Composer

**Version:** 0.1  
**Status:** Phase 3 — Production Core Complete  
**Tests:** 66 passing | **Rules:** 401 | **Pipeline:** 14 stages

---

## Quick Start

```bash
cargo test --workspace          # Rust tests
cd ui && npm install && npm run build
cd ../src-tauri && cargo run    # Launch desktop app
```

Or on Windows:
- **`dev.cmd`** — development with **frontend hot reload** (recommended while editing UI)
- **`start.cmd`** — full production build, then launch (no HMR)

```bash
# Dev: hot reload for Vue/CSS (run from repo root — Tauri finds src-tauri/)
cd ui && npm install && cd .. && ui/node_modules/.bin/tauri dev

# Production run (rebuild UI each time)
cd ui && npm run build && cd ../src-tauri && cargo run
```

### Keyboard shortcuts (piano roll)

| Shortcut | Action |
|----------|--------|
| Space | Play / stop (playhead returns to start position) |
| Ctrl+C / Ctrl+V | Copy / paste selected notes |
| Ctrl+B | Duplicate selected notes to next measure |
| Delete | Delete selected notes |

Tools: **Pointer**, **Box select**, **Brush** (add note), **Eraser**.

Playback uses **GM SoundFont** (FluidR3 via CDN on first play). Per-voice piano roll tabs filter Melody / Bass / Drums (drum map rows).

---

## What It Does

A **parameterized, explainable music composition engine**:

- Generates multi-voice scores (melody, counterpoint, bass, drums)
- Every note has **provenance** (rule ID, score, reason)
- **401 music theory rules** with beam search
- Export to **MIDI, MusicXML, ABC, SVG, PDF**
- **Full-score export**: multi-part MusicXML/ABC, correct clefs/key, drum unpitched notation
- **Vue 3 UI**: timeline, per-voice piano roll, inspector, GM SoundFont playback

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
