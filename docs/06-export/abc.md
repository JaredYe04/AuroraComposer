# ABC Notation Export Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Export Format Research Agent (ABC)  
**Dependencies:** `docs/02-music-model/ir.md` *(pending)*, [musicxml.md](musicxml.md), [ACAS v0.1](../00-overview/acas-v0.1.md)

---

## 1. Background

ABC notation is a plain-text format widely used in folk, traditional, and session music communities. Aurora Composer provides ABC export as a **lightweight sharing format** for simple scores — lead sheets, single-line melodies, and traditional tune exchange — not as a primary interchange format (see ADR-004).

```text
Music AST  →  Music IR  →  AbcExporter  →  .abc text file
```

ABC export targets human readability and web embedding (`abcjs`); it complements MusicXML for scenarios where compact text and quick preview matter more than notational completeness.

---

## 2. Existing Solutions

### 2.1 abcjs (JavaScript)

Browser renderer and player. Supports ABC 2.1 multi-voice via `V:` headers. Aurora frontend may use abcjs for optional ABC preview tab.

### 2.2 abcm2ps / abc2midi

Command-line tools converting ABC → PostScript/PDF/MIDI. Useful for validation golden files.

### 2.3 Music21 ABC Writer

Reference for note encoding (`C`, `^F`, `_B`, duration numbers). Aurora follows ABC 2.1 core with documented extensions.

### 2.4 EasyABC / The Session

Community corpora (thousands of tunes) demonstrate practical ABC scope: predominantly monophonic or 2–3 voice dance tunes.

---

## 3. Academic / Theoretical Foundation

ABC encodes **monophonic or limited polyphonic** music using ASCII characters:

- Pitch: letter A–G with octave modifiers (`C`, `c`, `c'`)
- Duration: numeric suffix or default unit from `L:` header
- Meter: `M:` header
- Key: `K:` header (required, typically last header before body)

ABC 2.1 formalizes **multi-voice** syntax:

```abc
X:1
T:Title
M:4/4
L:1/8
K:C
V:1
CDEF | GABc |
V:2
E2 G2 | c2 B2 |
```

Voice independence is **limited** compared to MusicXML: no per-voice dynamics, complex cross-staff layout, or orchestral part extraction.

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| **Correctness** | Excellent for 1–2 voice diatonic melodies; degrades beyond 4 voices |
| **Controllability** | `export.abc.max_voices`, `export.abc.mode` (melody-only default) |
| **Explainability** | Provenance not representable — document as limitation |
| **Performance** | Trivial (<10 ms) |
| **Extensibility** | `%%` comment directives for Aurora metadata (non-standard) |
| **Complexity** | Low — ~600 LOC |

---

## 5. Comparison of Approaches

### 5.1 ABC vs MusicXML for Aurora

| Aspect | ABC | MusicXML |
|--------|-----|----------|
| File size | Small (KB) | Large (100KB+) |
| Human editable | Yes | No |
| Multi-part orchestral | Poor | Excellent |
| Chord symbols | `w:` field (limited) | `<harmony>` rich |
| Provenance | Not supported | `aurora:*` namespace |
| Web preview | abcjs native | Verovio WASM |
| Tool ecosystem | Folk/trad | Professional notation |

**When to use ABC:**

- Sharing a **single melody line** or simple lead sheet
- Embedding in web pages or README documentation
- Quick copy-paste into The Session / folk forums
- Lightweight email attachment

**When to use MusicXML instead:**

- Multi-voice counterpoint (>2 independent parts)
- Full score export with dynamics, articulations, form
- Handoff to MuseScore/Dorico
- Preserving provenance or generation parameters
- Drum kit notation with unpitched display

### 5.2 Export Scope Modes

| Mode | Voices | Output |
|------|--------|--------|
| `melody_only` | 1 (primary melody voice) | Single voice ABC |
| `lead_sheet` | 1 + chord symbols | `w:` chord annotations |
| `multi_voice` | Up to 4 | `V:` headers |
| `full` | All (with warnings) | May produce unreadable ABC |

**Default:** `melody_only`. UI warns when user selects ABC for >2 voices.

---

## 6. Recommended Solution

### 6.1 ABC Version and Compliance

- **Standard:** ABC 2.1 (2011)
- **Encoding:** UTF-8; ASCII-safe fallback for non-Latin titles (`T:` transliteration)
- **Line endings:** LF (`\n`)
- **Required headers:** `X:`, `T:`, `M:`, `L:`, `K:` (in conventional order)

### 6.2 Scope Limitations (Normative)

| Feature | ABC Support | Aurora Behavior |
|---------|-------------|-----------------|
| Max voices (default) | 1 | Configurable to 4 |
| Max voices (hard limit) | 4 | Error `E_ABC_VOICE_OVERFLOW` |
| Chord symbols | `w:` lines | Lead sheet mode only |
| Dynamics | `!p!`, `!f!` annotations | Common symbols only |
| Articulations | `!.!`, `!->!` etc. | Subset |
| Tuplets | ` (3` syntax | Simple triplets |
| Grace notes | `{c}` | Supported |
| Drums | `_` note mapping (non-standard) | **Excluded** — use MIDI |
| Provenance | `%%aurora-provenance` comment | Optional debug, not parsed on import |
| Form / sections | `|:`, `:` repeat bars | Repeat only; no section roles |
| Tempo changes | `Q:` | First tempo only in v0.1 |
| Key changes | `V:` voice key or `[K:]` mid-tune | Mid-tune `[K:]` supported |

---

## 7. Architecture

```text
┌─────────────────────────────────────────────────────────┐
│                  ABC Export Pipeline                     │
├─────────────────────────────────────────────────────────┤
│  Music IR + AST metadata                                 │
│       │                                                  │
│       ▼                                                  │
│  AbcScopeSelector (melody / lead_sheet / multi_voice)    │
│       │                                                  │
│       ▼                                                  │
│  IrToAbcMapper                                           │
│    ├── HeaderBuilder (X, T, M, L, K, Q)                  │
│    ├── VoiceSplitter (max 4)                             │
│    ├── NoteEncoder (pitch + duration)                    │
│    ├── ChordSymbolWriter (w: lines)                      │
│    └── BarlineInserter (| |] |:)                          │
│       │                                                  │
│       ▼                                                  │
│  String (.abc)                                           │
└─────────────────────────────────────────────────────────┘
```

No ABC import in v0.1 — users import via MusicXML.

---

## 8. Data Structures

### 8.1 AbcExportConfig

```rust
pub struct AbcExportConfig {
    pub mode: AbcExportMode,     // MelodyOnly | LeadSheet | MultiVoice | Full
    pub max_voices: u8,          // default 1, max 4
    pub default_note_length: Rational, // maps to L: header
    pub include_chord_symbols: bool,
    pub include_annotations: bool, // dynamics, articulations
    pub tune_number: u32,        // X: header
    pub aurora_comments: bool,   // %% metadata comments
}

pub enum AbcExportMode {
    MelodyOnly,
    LeadSheet,
    MultiVoice,
    Full,
}
```

### 8.2 ABC Document Structure

```text
AbcDocument
├── headers: Vec<AbcHeader>       // X, T, C, M, L, Q, K
├── voices: Vec<AbcVoice>         // optional V: definitions
└── body: Vec<AbcLine>            // bar-separated note strings
```

---

## 9. Algorithms

### 9.1 Note Encoding

ABC pitch mapping from MIDI note number or AST pitch:

```text
Middle C (MIDI 60) → C
One octave up → c
Two octaves up → c'
One octave down → C,

Accidentals:
  sharp → ^  (e.g., ^F)
  flat  → _  (e.g., _B)
  natural → = (e.g., =C)
```

Duration encoding relative to `L:` unit:

```text
L:1/8 (default eighth)
quarter note → C2
half note → C4
dotted quarter → C3
sixteenth → C/2
```

### 9.2 Multi-Voice Body Generation

```
function export_abc_multi(ir: MusicIR, config: AbcExportConfig) -> String:
    voices = select_voices(ir, config.max_voices)
    doc = build_headers(ast.metadata, ir.global_attributes)

    for i, voice in enumerate(voices):
        doc.add_header("V:" + str(i+1))
        doc.add_header("  name=" + voice.name)

    // Merge by measure for aligned barlines
    for measure_index in range(ir.measure_count):
        for i, voice in enumerate(voices):
            notes = encode_measure(voice, measure_index)
            doc.add_line("V:" + str(i+1))
            doc.add_line(notes + " |")

    return doc.serialize()
```

### 9.3 Chord Symbol Lines (Lead Sheet Mode)

From IR harmony events or AST `Event::Chord`:

```abc
w: Am | D7 | G | C |
```

Placed above corresponding bar in `w:` field (ABC 2.1 chord syntax). Complex jazz symbols (e.g., `Dbmaj7#11`) may simplify to `Dbmaj7` with warning.

### 9.4 Repeat and Barline Mapping

| AST / IR | ABC |
|----------|-----|
| Measure boundary | `\|` |
| Final barline | `\|]` |
| Repeat start | `\|:` |
| Repeat end | `:|\|` |
| Double bar | `\|\|` |

Section roles (verse, chorus) **not encoded** — use MusicXML `aurora:section-role`.

---

## 10. Interfaces

### 10.1 Rust API

```rust
pub fn export_abc(
    ast: &Composition,
    ir: &MusicIR,
    config: &AbcExportConfig,
) -> Result<String, ExportError>;

pub fn abc_voice_count_warning(ast: &Composition) -> Option<VoiceOverflowWarning>;
```

### 10.2 Tauri IPC

```typescript
export_composition_abc(composition_id: string, config: AbcExportConfig): Promise<string>
```

### 10.3 Frontend abcjs Integration

```typescript
// Optional ABC preview tab
import ABCJS from 'abcjs';
ABCJS.renderAbc('abc-paper', abcString, { responsive: 'resize' });
```

---

## 11. Parameter Mappings

### 11.1 AST / IR → ABC Mapping Table

| AST / IR Entity | ABC Construct | Notes |
|---------------|---------------|-------|
| `Composition.metadata.title` | `T:` | Required |
| `Composition.metadata.composer` | `C:` | Optional |
| `GlobalAttributes.time_signature` | `M:` | e.g. `M:4/4` |
| `GlobalAttributes.key` | `K:` | e.g. `K:Am` |
| `GlobalAttributes.tempo` | `Q:` | e.g. `Q:1/4=120` |
| `Voice` (primary) | Body notation (no V: if single) | Melody mode |
| `Voice` (2–4) | `V:1`, `V:2`, ... | Multi-voice mode |
| `Event::Note.pitch` | Letter + accidental | See §9.1 |
| `Event::Note.duration` | Duration suffix | Relative to `L:` |
| `Event::Rest` | `z` + duration | e.g. `z2` |
| `Event::Chord.symbol` | `w:` chord line | Lead sheet mode |
| `Measure.repeat_start` | `\|:` | |
| `Measure.repeat_end` | `:|` | |
| `Event::Note.grace` | `{note}` | |
| `Event::Note.tuplet` | `(3` prefix | Triplets default |
| `Event::Note.staccato` | `!.!` | Annotation |
| `Event::Note.provenance` | `%%aurora-id:uuid` | Comment only |
| `Voice[drums]` | **Excluded** | Recommend MIDI |
| `Section.role` | **Not mapped** | — |
| `Phrase` | **Not mapped** | — |

### 11.2 User Parameters

| Parameter | Effect |
|-----------|--------|
| `export.abc.mode` | Scope selection |
| `export.abc.max_voices` | Voice cap |
| `theme.count` | If >1 theme, multi_voice may help |
| `voice.count` | Triggers overflow warning if > max_voices |
| `harmony.complexity` | Affects `w:` chord symbol density |

---

## 12. Explainability Model

ABC export does not support structured provenance. Optional debug mode emits comments:

```abc
%%aurora-version: 0.1
%%aurora-generated: 2026-07-04T12:00:00Z
%%aurora-params-sha256: abc123...
```

Per-note provenance as `%%aurora-id` comments is **not recommended** for production (file bloat). Explainability remains in-app via AST; users needing provenance export must use MusicXML interchange profile.

---

## 13. Future Expansion

| Phase | Feature |
|-------|---------|
| v0.2 | ABC import (melody-only, no provenance) |
| v0.2 | `%%score` brace grouping for piano grand staff |
| v0.3 | Integrated abcjs playback in Vue UI |
| v0.4 | ABC 2.2 features as standard evolves |

---

## 14. Open Questions

1. Include `C:` composer header from metadata by default?
2. Should lead sheet mode export bass line as second voice or chord roots only?
3. Transposing instruments (`K:` + `V:` transposition) — Phase 1 scope?
4. abcjs bundle size vs lazy load for optional preview tab?

---

## 15. References

- ABC Notation Standard 2.1 (2011)
- abcjs documentation: https://www.abcjs.net/
- abcm2ps user guide
- Music21 ABC writer module
- [MusicXML Specification](musicxml.md) — primary format
- [MIDI Export Specification](midi.md)
- [ADR-004](../../decisions/ADR-004-musicxml-primary-export.md)
- [Export Research Notes](../../research/export-research-notes.md)

---

*End of ABC Notation Export Specification v0.1*
