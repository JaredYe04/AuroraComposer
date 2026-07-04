# Plugin & UI Research Notes

**Date:** 2026-07-04  
**Agent:** Plugin & UI Research Agent  
**Status:** Raw research — feeds into `docs/07-plugin/` and `docs/08-ui/`

---

## Plugin Ecosystem Survey

### Existing Plugin Models

| System | Model | Strengths | Weaknesses for Aurora |
|--------|-------|-----------|----------------------|
| **VST/AU** | Binary DSP in host process | Mature audio ecosystem | No AST, no explainability, audio-only |
| **LV2** | RDF manifest + shared lib | Open standard | Audio/MIDI focused, no composition semantics |
| **CLAP** | Modern plugin ABI | Clean extension model | Same audio limitation |
| **Obsidian plugins** | JS sandbox in Electron | Easy distribution | No performance for search |
| **VS Code extensions** | Node worker + capability API | Sandboxing precedent | Not Rust-native |
| **Tauri plugins** | Rust crate + capability manifest | Native IPC, capability model | App-level, not user plugins |
| **WASM modules** | Portable sandbox | Memory-safe isolation | FFI overhead, no direct Rust trait impl |
| **Rhythmic / OpenMusic** | Lisp libraries | Algorithmic composition | Closed, no provenance |

### Key Finding

No existing plugin system combines **AST mutation**, **rule-based provenance**, and **desktop sandboxing**. Aurora must define a hybrid:

1. **Native Rust plugins** (`.dll`/`.so`/`.dylib`) for performance-critical harmony/rhythm engines
2. **WASM plugins** for untrusted third-party and AI inference wrappers
3. **Manifest-driven capabilities** aligned with Tauri 2.x permission model

---

## Sandboxing Research

### Tauri 2.x Capabilities

Tauri 2 uses capability files (`capabilities/*.json`) to grant scoped permissions:

- `fs:read` / `fs:write` — path-restricted
- `http:request` — domain allowlist (AI plugins only, user opt-in)
- `shell:execute` — disabled by default

**Implication:** Plugin Host wraps each plugin in a capability profile derived from manifest `permissions[]`.

### WASM Sandbox Options

| Runtime | Pros | Cons |
|---------|------|------|
| **wasmtime** | Rust-native, WASI | No direct AST struct access |
| **wasmer** | Good perf | Similar FFI limits |
| **Extism** | Plugin SDK with host functions | Extra dependency |

**Recommendation:** Host provides `extism`-style host functions: `read_ast_region`, `write_patch`, `log`, `get_param`. AST serialized as CBOR across boundary.

### Native Dynamic Library Risks

- Memory unsafety in plugin code can crash host
- Mitigation: subprocess isolation for untrusted plugins (Phase 3)
- v0.1: signed plugins + capability restrictions + crash recovery

---

## UI Framework Survey

### Music Editor UIs

| Tool | Timeline | Piano Roll | Provenance | Tech |
|------|----------|------------|------------|------|
| **Ableton Live** | Clip lanes | MIDI editor | None | C++ |
| **FL Studio** | Pattern blocks | Piano roll | None | Delphi/C++ |
| **MuseScore** | Linear score | N/A (notation) | None | Qt/C++ |
| **Lenardo** | Section blocks | Note grid | None | Web |
| **OpenMusic** | Patch cords | Score viewer | None | Lisp |
| **AIVA Studio** | Section timeline | Limited | Black box | Web |

**Gap:** No commercial tool exposes rule-level provenance on note click. Aurora's Inspector is a differentiator.

### Vue 3 Music UI Libraries

| Library | Use Case | Verdict |
|---------|----------|---------|
| **Tone.js** | WebAudio playback | Selected for preview |
| **Verovio WASM** | MusicXML → SVG | Selected for notation |
| **wavesurfer.js** | Waveform | Optional for audio import |
| **Custom Canvas/SVG** | Piano roll | Required — no off-shelf provenance-aware editor |

### Piano Roll Prior Art

- **Magenta Studio** (TensorFlow): piano roll in browser, no explainability
- **Signal** (partwise): open-source JS piano roll — reference for grid math
- **DAWproject** format: interchange, not UI

Grid math: beats × pitch classes, snap to subdivision from `rhythm.subdivision` parameter.

---

## Tauri IPC Patterns

### Command Categories (from architecture review)

1. **Project CRUD** — create, open, save, close
2. **Composition** — get AST, apply patch, validate
3. **Generation** — start job, cancel, get status
4. **Export** — project to IR, export format
5. **Plugins** — list, install, enable, configure
6. **Provenance** — get chain for event

### Async Job Pattern

```
Frontend                    Tauri Backend
   │                              │
   │── generate_composition ─────►│ spawn tokio task
   │◄── job_id ───────────────────│
   │                              │ ... pipeline runs ...
   │◄── event: job-progress ──────│ emit per stage
   │◄── event: job-complete ──────│ emit with composition handle
```

Cancellation: `cancel_job(job_id)` sets atomic flag checked between pipeline stages.

### Serialization

- IPC: JSON (human-debuggable, serde-compatible)
- Project files: CBOR (smaller, faster)
- Large AST transfers: optional chunked `get_composition_slice(node_id)`

---

## Provenance UI Research

### Philosophy Requirement

> Clicking any note shows: Generated by: Harmony Rule #42, Score: +13, Reason: Passing Tone

### Display Hierarchy (user-tested patterns from XAI literature)

1. **Primary line** — one sentence summary (always visible)
2. **Rule list** — expandable, sorted by score contribution
3. **Search context** — step, beam rank, accumulated score
4. **Parameter snapshot** — relevant params at generation time
5. **Parent chain** — link to source event (repair, transform, manual edit)

### Hover vs Click

- **Hover (piano roll):** tooltip with primary line only (performance)
- **Click:** full Inspector panel with complete chain
- **Keyboard:** `I` focuses Inspector when event selected

---

## Plugin Distribution

### Manifest Discovery Paths

```
{app_data}/plugins/          — user-installed
{app_resources}/plugins/     — bundled defaults
{project_dir}/.aurora/plugins/ — project-local overrides
```

### Versioning

Semantic versioning on plugin + `api_version` compatibility field matching Aurora engine version.

### Signing (Phase 3)

Code signing for marketplace; v0.1 accepts unsigned local plugins with warning.

---

## Open Research Items

| ID | Topic | Next Step |
|----|-------|-----------|
| R-PU-1 | WASM vs native default for third-party | ADR-005 |
| R-PU-2 | Pinia vs composable-only state | Recommend Pinia for job/AST handles |
| R-PU-3 | Piano roll Canvas vs SVG | Canvas for >500 events visible |
| R-PU-4 | Plugin hot-reload during dev | SDK `aurora-plugin dev --watch` |
| R-PU-5 | Naive UI vs custom design system | Naive UI for v0.1 shell |

---

## References (External)

- Tauri 2 Capabilities: https://v2.tauri.app/security/capabilities/
- Extism plugin SDK: https://extism.org/
- CLAP plugin spec: https://github.com/free-audio/clap
- Samek et al. (2019) — XAI visualization patterns
- Verovio: https://www.verovio.org/
- Tone.js: https://tonejs.github.io/
