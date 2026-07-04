# Aurora Composer — Progress Tracker

**Last Updated:** 2026-07-04  
**Phase:** 3 — Production (Feature Complete for v0.1)  
**Design Freeze:** ✅ [design-freeze-v0.1.md](reviews/design-freeze-v0.1.md)

---

## Overall Progress

| Metric | Value |
|--------|-------|
| Phase 1 specifications | **49 / 49** ✅ |
| ADRs accepted | **8** |
| Rust workspace tests | **73 / 73 passing** ✅ |
| Rule catalog | **401 rules** registered |
| Pipeline stages | **14 / 14** ✅ |
| Export formats | MIDI, MusicXML, ABC, SVG preview, PDF bytes |
| Plugin host | 3 built-in style + AI stub + WASM discovery |
| UI components | Timeline, Piano Roll, Inspector, Player, Score, Project, Plugins |

---

## Phase Status

| Phase | Status |
|-------|--------|
| Phase 1: Architecture Design | ✅ Complete |
| Phase 2: Prototype | ✅ Complete |
| Phase 3: Production | ✅ **Feature Complete (v0.1)** |

---

## Implementation Matrix

| Component | Status | Tests |
|-----------|--------|-------|
| `aurora-core` | ✅ | 8 |
| `aurora-ast` | ✅ | 23 (patch + project) |
| `aurora-rules` | ✅ | 13 (401 rules, category evaluators) |
| `aurora-plugin` | ✅ | 7 (WASM loader + AI stub) |
| `aurora-engine` | ✅ | 13 (full 14-stage pipeline) |
| `aurora-export` | ✅ | 9 (PDF + SVG) |
| `src-tauri/` | ✅ | IPC: project, patch, plugins, PDF |
| `ui/` | ✅ | Score viewer, project menu, piano roll edit |

---

## Capabilities

### Generation Pipeline (14 stages)

Style → Emotion → Structure → Theme → Phrase hooks → Harmony → Rhythm → Melody → Counterpoint → Bass → Drums → Decoration → Repair → Validation → Export

### Multi-voice output

- Melody (beam search)
- Alto/counterpoint (optional, beam search)
- Bass (beam search)
- Drums (channel 10 patterns)
- Full provenance on every event

### Export

| Format | Command / UI |
|--------|--------------|
| MIDI Type 1 | `export_midi` |
| MusicXML 4.0 | `export_musicxml` |
| ABC full score (multi-voice) | `export_abc` (Full mode) |
| MusicXML full score | `export_musicxml` (per-part clefs, drum unpitched) |
| SVG score preview | `export_svg_preview` |
| PDF bytes (backend) | `export_pdf_bytes` |
| In-app score | ABC (abcjs), MusicXML/PDF (Verovio WASM); auto-refresh on edit |
| Tauri file save | MIDI / MusicXML / ABC / PDF via native save dialog |

### UI

- 3-column workspace (parameters / timeline+piano roll / inspector+score); **right panel defaults to 50% width**
- **VoiceSwitcher**: per-voice piano roll (Melody / Bass / Drums drum-map rows)
- **ScoreViewer**: full-score MusicXML → Verovio; revision sync on patch
- **ProjectMenu**: New / Load / Save `.aurora` (CBOR)
- **PluginMenu**: plugins in header dropdown (removed from left sidebar)
- **PianoRoll**: pointer / box / brush / eraser tools; drag pitch; insert/delete notes
- Keyboard: Space play/stop (restore playhead), Ctrl+C/V/B, Delete
- **GM SoundFont playback** (soundfont-player + Tone.Part); seek/reschedule
- Provenance tooltip on hover, full chain in inspector
- Light/dark theme CSS variables on menus and timeline

### Plugins

- `com.aurora.plugins.classical-style`
- `com.aurora.plugins.jazz-style`
- `com.aurora.plugins.pop-style`
- `com.aurora.plugins.ai-stub` (emotion → weight deltas)
- External WASM: manifest scan + register (execution deferred to wasmtime)

### Rules

- 401 rules registered with unique IDs
- 30+ full evaluators in `implemented.rs`
- Category-aware soft/hard evaluators for remaining stubs via `helpers.rs`

---

## Run Commands

```powershell
# All tests (73)
cargo test --workspace

# Desktop app — rebuild UI first, then run
cd ui; npm install; npm run build
cd ../src-tauri; cargo run
```

> **Note:** `cargo run` in dev mode uses the Vite dev server (`http://localhost:5173`). After UI changes, restart or let HMR reload. For production bundle, run `npm run build` in `ui/` before `cargo run --release`.

---

## Known Limitations (Post v0.1)

- WASM plugin **execution** not yet wired (wasmtime); discovery/registration only
- PDF backend export is metadata placeholder; in-app PDF uses Verovio SVG → jsPDF
- Individual theory rules beyond category stubs not fully hand-implemented

---

## Changelog

| Date | Event |
|------|-------|
| 2026-07-04 | Phase 1 Design Freeze |
| 2026-07-04 | Phase 2 prototype (52 tests) |
| 2026-07-04 | Phase 3: full pipeline, 401 rules, plugins, UI, export — **66 tests** |
| 2026-07-04 | Phase 3+ : score viewer, project I/O, piano roll patch, plugins UI, white-screen fix — **73 tests** |
