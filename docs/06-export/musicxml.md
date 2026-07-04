# MusicXML Export/Import Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Export Format Research Agent (MusicXML)  
**Dependencies:** `docs/02-music-model/ast.md` *(pending)*, `docs/02-music-model/ir.md` *(pending)*, [ACAS v0.1](../00-overview/acas-v0.1.md), [ADR-004](../../decisions/ADR-004-musicxml-primary-export.md)

---

## 1. Background

MusicXML is the de facto standard for symbolic music notation interchange. Aurora Composer treats MusicXML 4.0 as the **primary export and interchange format** (ADR-004): the canonical artifact users exchange with MuseScore, Dorico, Sibelius, and academic tooling, and the input to the PDF rendering pipeline.

The export path is:

```text
Music AST  →  (optional normalize)  →  Music IR  →  MusicXML Serializer  →  .musicxml file
MusicXML file  →  MusicXML Parser  →  AST Builder  →  Music AST
```

MusicXML export must preserve **notational semantics** (pitch, rhythm, articulation, dynamics, multi-part layout) and **Aurora-specific provenance** via a registered XML namespace extension. Import must reconstruct a valid AST sufficient for re-generation, editing, and explainability display.

This specification defines:

- AST/IR node → MusicXML element mapping (normative table)
- Partwise vs timewise format choice
- Provenance metadata extension strategy
- Round-trip fidelity requirements (Tier A / B / C)
- Serializer and parser architecture

---

## 2. Existing Solutions

### 2.1 Music21 (Python)

Music21's `musicxml.m21ToXml` and `musicxml.xmlToM21` modules are the most complete open-source reference. Strengths: broad MusicXML 3/4 coverage, chord symbols, ornaments, multi-staff parts. Weaknesses: no generation provenance, Python-only, performance limits on large scores.

**Aurora adoption:** Mirror Music21's partwise structure and `<chord/>` stacking convention; extend with Aurora namespace attributes.

### 2.2 MuseScore / LibreScore

MuseScore 4 reads/writes MusicXML 4.0 with minor vendor extensions (`<museScore>` tags). Export compatibility target: MuseScore 4 opens Aurora MusicXML without data loss for Tier A elements.

### 2.3 Verovio

Verovio consumes MusicXML for SVG engraving. Aurora PDF preview path depends on MusicXML subset compatibility (see [pdf.md](pdf.md)). Avoid Verovio-unsupported constructs in default export profile.

### 2.4 Rust Ecosystem

No mature MusicXML 4.0 Rust crate exists as of 2026. Aurora will implement a dedicated serializer/parser in the Rust core, potentially splitting:

- `aurora-musicxml` — low-level DOM + validation
- `aurora-export-musicxml` — AST/IR mapping layer

Reference: `quick-xml` for streaming serialization; optional `roxmltree` for import.

### 2.5 Commercial SDKs

MusicXML official SDK (C++) exists but adds build complexity. Not recommended for Aurora Tauri stack.

---

## 3. Academic / Theoretical Foundation

MusicXML encodes **Common Western Notation (CWN)** as defined by W3C Music Notation Community Group. The representation model aligns with:

- **Lerdahl & Jackendoff** hierarchical grouping — maps to Aurora Section/Phrase via `<direction>` and `<barline>`
- **Metric hierarchy** — `<divisions>` + duration in divisions per quarter
- **Harmonic analysis** — `<harmony>` elements with root, kind, bass, degrees (pop/jazz lead-sheet and classical Roman numerals via `<function>`)

MusicXML does **not** encode:

- Generative algorithm state
- Constraint satisfaction traces
- Theme/motif identity graphs

These require Aurora extension attributes or companion sidecar files (discouraged; prefer inline namespace).

---

## 4. Engineering Analysis

| Criterion | Assessment |
|-----------|------------|
| **Correctness** | MusicXML 4.0 schema validates structural correctness; semantic correctness requires Aurora mapping tests |
| **Controllability** | Export profiles (`compact`, `full`, `interchange`) control verbosity and extension inclusion |
| **Explainability** | Provenance preserved via `aurora:provenance` attribute (JSON-encoded) on events |
| **Performance** | Streaming serializer target: <500 ms for 500-measure score on desktop CPU |
| **Extensibility** | Namespace extension pattern; ExportPlugin may inject `<direction>` elements |
| **Complexity** | High — full parser is ~3–5k LOC; serializer ~2k LOC; phased delivery |

### 4.1 Risk Register

| Risk | Mitigation |
|------|------------|
| Third-party MusicXML quirks | Liberal import parser; strict export validator |
| Large provenance attributes | Compress JSON; optional `provenance="ref:uuid"` with sidecar |
| Tuplet edge cases | Golden-file tests from Music21 corpus |
| Drum unpitched mapping | GM percussion table (see §11.3) |

---

## 5. Comparison of Approaches

### 5.1 Partwise vs Timewise

| Aspect | Partwise (`score-partwise`) | Timewise (`score-timewise`) |
|--------|----------------------------|----------------------------|
| Structure | `<part>` → `<measure>*` | `<measure>` → `<part>*` |
| Aurora fit | Natural: Voice/Part = `<part>` | Requires measure-synchronized merge |
| Tool support | Dominant default | Less common |
| Multi-voice export | One part per voice or staff | All parts per measure |
| Import complexity | Sequential per-part parsing | Must synchronize measure indices |
| Verovio/MuseScore | Native | Converted internally |

**Recommendation:** Partwise canonical (see §6).

### 5.2 Direct AST vs IR Serialization

| Approach | Pros | Cons |
|----------|------|------|
| AST → MusicXML direct | Preserves Section/Phrase markers inline | Hierarchical walk complexity |
| IR → MusicXML | Flattened, exporter-friendly | Loses form hierarchy unless IR carries markers |

**Recommendation:** IR-primary serialization with `FormMarker` IR events for section boundaries; AST consulted for metadata and `<credit>`.

### 5.3 Provenance: Sidecar vs Inline

| Approach | Pros | Cons |
|----------|------|------|
| Inline `aurora:*` attributes | Single file; round-trip | Non-standard; large attributes |
| JSON sidecar `.aurora.json` | Clean MusicXML | Two files; user confusion |
| `<miscellaneous>` hack | No namespace | Overloaded; parser ignores |

**Recommendation:** Inline namespace attributes with optional compact mode (see §8.3).

---

## 6. Recommended Solution

### 6.1 Format Choice

- **Document type:** `score-partwise` version 4.0
- **Encoding:** UTF-8, no BOM
- **File extension:** `.musicxml` (uncompressed XML) or `.mxl` (ZIP container with `META-INF/container.xml` — Phase 2)
- **Doctype:** `<!DOCTYPE score-partwise PUBLIC "-//Recordare//DTD MusicXML 4.0 Partwise//EN" "http://www.musicxml.org/dtds/partwise.dtd">` optional; schema validation preferred

### 6.2 Round-Trip Fidelity Tiers

#### Tier A — Notational Core (MUST preserve Aurora ↔ MusicXML ↔ Aurora)

| Category | Elements |
|----------|----------|
| Pitch | `<pitch>`, `<unpitched>`, `<rest>`, `<alter>`, microtones via `<alter>` + `<accidental>` |
| Rhythm | `<duration>`, `<type>`, `<dot>`, `<time-modification>`, `<tuplet>` |
| Meter | `<time>`, `<sensory-music>` N/A |
| Key | `<key>`, `<key>` mode, `<cancel>` |
| Clef | `<clef>`, `<clef>` ottava via `<transpose>` |
| Multi-part | Part count, part names, MIDI channel/instrument |
| Ties/slurs | `<tie>`, `<slur>`, `<beam>` |
| Articulations | `<articulations>`, `<ornaments>`, `<technical>` |
| Dynamics | `<dynamics>`, `<wedge>` |
| Tempo | `<sound tempo="">`, `<metronome>` |
| Repeats | `<repeat>`, `<barline>`, `<ending>` |
| Chord symbols | `<harmony>` root/kind/bass/degree |

#### Tier B — Aurora Semantic Extensions (MUST preserve Aurora ↔ Aurora; MAY strip by third parties)

| Category | Mechanism |
|----------|-----------|
| Provenance chain | `aurora:provenance` attribute |
| Event UUID | `aurora:id` attribute |
| Theme/motif ref | `aurora:theme-ref` attribute |
| Section role | `aurora:section-role` on `<direction>` |
| Generation parameters | `<credit>` + `aurora:params` in `<identification>` |
| Voice group | `aurora:voice-group` on `<score-part>` |

#### Tier C — Engraving / Layout (BEST EFFORT)

| Category | Elements |
|----------|----------|
| System breaks | `<print new-system="">` |
| Page breaks | `<print new-page="">` |
| Staff layout | `<staff-layout>`, `<staff-details>` |
| Positioning | default-x/y, relative-x/y |

**Round-trip test contract:**

```
assert roundtrip(composition) satisfies:
  ∀ event e in original.events:
    pitch(e) = pitch(e')
    duration(e) = duration(e')
    voice(e) = voice(e')
    provenance(e) = provenance(e')  // Tier B
  section_tree(original) ≅ section_tree(result)  // via markers
```

Third-party import (MuseScore file → Aurora):

```
assert import(museScoreXml) satisfies Tier A only
provenance(e) = manual_import stub
```

---

## 7. Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    Export Pipeline                           │
├─────────────────────────────────────────────────────────────┤
│  Composition AST                                             │
│       │                                                      │
│       ▼                                                      │
│  AstToIrProjector ──────────────────► Music IR Snapshot      │
│       │                                      │               │
│       │ (metadata, credits)                  ▼               │
│       │                              IrToMusicXmlMapper        │
│       │                                      │               │
│       └────────── ProvenanceInjector ◄───────┘               │
│                      │                                       │
│                      ▼                                       │
│              MusicXmlDocument (DOM)                          │
│                      │                                       │
│                      ▼                                       │
│              XmlSerializer (streaming)                       │
│                      │                                       │
│                      ▼                                       │
│                 .musicxml / .mxl                             │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    Import Pipeline                           │
├─────────────────────────────────────────────────────────────┤
│  .musicxml → XmlParser → MusicXmlDocument                    │
│       → MusicXmlValidator (schema subset)                    │
│       → MusicXmlToAstMapper                                  │
│       → ProvenanceExtractor (aurora:* attrs)                 │
│       → Composition AST                                      │
└─────────────────────────────────────────────────────────────┘
```

### 7.1 Export Profiles

| Profile | Tier B | Tier C | Use Case |
|---------|--------|--------|----------|
| `interchange` | ✓ | ✗ | Aurora project save, re-import |
| `publish` | ✗ | ✓ | MuseScore/Dorico handoff |
| `preview` | ✗ | ✗ | Verovio minimal subset |
| `full` | ✓ | ✓ | Debug / archival |

Default user export: `interchange`. PDF path uses `preview` or `publish` depending on quality setting.

---

## 8. Data Structures

### 8.1 MusicXmlDocument (Internal DOM)

```rust
/// Root document — partwise only in v0.1
struct MusicXmlDocument {
    version: String,           // "4.0"
    identification: Identification,
    defaults: Defaults,
    credit: Vec<Credit>,
    part_list: PartList,
    parts: Vec<Part>,
    aurora_extensions: AuroraExtensions,
}

struct Part {
    id: String,                // "P1", "P2", ...
    measures: Vec<Measure>,
}

struct Measure {
    number: u32,
    implicit: bool,
    width: Option<f64>,
    attributes: Option<Attributes>,
    notes: Vec<NoteEvent>,
    directions: Vec<Direction>,
    harmonies: Vec<Harmony>,
    barline: Option<Barline>,
    print: Option<Print>,
}

struct NoteEvent {
    // Unified note/rest/unpitched
    element_type: NoteElementType,
    duration: i32,             // in divisions
    voice: u8,
    staff: Option<u8>,
    chord: bool,               // <chord/> flag
    pitch: Option<Pitch>,
    rest: Option<Rest>,
    unpitched: Option<Unpitched>,
    tie: Vec<Tie>,
    notations: Option<Notations>,
    aurora_id: Option<Uuid>,
    aurora_provenance: Option<ProvenancePayload>,
}
```

### 8.2 Divisions and Duration

MusicXML expresses note length in **divisions** (per quarter note). Aurora internal time uses **rational ticks** (see pending `timeline.md`).

```text
divisions = export_config.divisions_per_quarter  // default 480
musicxml_duration = ir_event.duration_ticks * divisions / ir.ticks_per_quarter
```

Tuplet mapping:

```xml
<time-modification>
  <actual-notes>3</actual-notes>
  <normal-notes>2</normal-notes>
  <normal-type>eighth</normal-type>
</time-modification>
<notations>
  <tuplet type="start" bracket="yes"/>
</notations>
```

### 8.3 Provenance Metadata Extension Strategy

**Namespace:**

```xml
xmlns:aurora="http://aurora-composer.dev/ns/v1"
```

**Registration:** Document in `identification/miscellaneous`:

```xml
<identification>
  <encoding>
    <software>Aurora Composer 0.1</software>
    <encoding-date>2026-07-04</encoding-date>
  </encoding>
  <miscellaneous>
    <miscellaneous-field name="aurora-namespace">http://aurora-composer.dev/ns/v1</miscellaneous-field>
    <miscellaneous-field name="aurora-version">1</miscellaneous-field>
  </miscellaneous>
</identification>
```

**Attribute schema (v1):**

| Attribute | Element | Type | Description |
|-----------|---------|------|-------------|
| `aurora:id` | note, rest, harmony | UUID string | Stable event identifier |
| `aurora:provenance` | note, rest, harmony | JSON (escaped) | Provenance chain |
| `aurora:theme-ref` | note | string | Theme slot ID |
| `aurora:section-role` | direction | enum string | verse, chorus, bridge, ... |
| `aurora:voice-group` | score-part | string | e.g. "rhythm_section" |
| `aurora:rule-id` | note | string | Last applied rule (compact mode) |

**Provenance JSON payload (inline):**

```json
{
  "v": 1,
  "origin": "generation",
  "stage": "melody",
  "rule_ids": ["VL-001", "HR-042"],
  "eval_score": 0.847,
  "search_step": 1247,
  "parent_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

**Compact mode:** When `export.profile = publish`, emit only `aurora:rule-id` or omit entirely.

**Validation:** Non-Aurora parsers ignore unknown namespaced attributes per XML Namespaces spec. Aurora import requires namespace declaration for Tier B restore; missing namespace → Tier A only with `provenance: manual_import`.

### 8.4 Part List and Instrument Mapping

```xml
<part-list>
  <score-part id="P1">
    <part-name>Melody</part-name>
    <part-abbreviation>Mel.</part-abbreviation>
    <score-instrument id="P1-I1">
      <instrument-name>Piano</instrument-name>
    </score-instrument>
    <midi-device id="P1-M1"/>
    <midi-instrument id="P1-I1">
      <midi-channel>1</midi-channel>
      <midi-program>1</midi-program>
    </midi-instrument>
  </score-part>
</part-list>
```

---

## 9. Algorithms

### 9.1 IR-to-MusicXML Serialization (Outline)

```
function export_musicxml(ir: MusicIR, ast: Composition, config: ExportConfig) -> String:
    doc = new MusicXmlDocument(version="4.0")
    doc.identification = map_metadata(ast.metadata)
    doc.part_list = build_part_list(ir.parts)
    doc.defaults = build_defaults(config.divisions)

    for each ir_part in ir.parts:
        part = new Part(id=ir_part.id)
        current_divisions = config.divisions
        measure_buffer = []

        for each ir_event in ir_part.events_ordered():
            if ir_event is FormMarker:
                flush_measure_buffer()
                part.measures.add(direction_from_marker(ir_event))
                continue

            while measure_buffer.duration < ir_event.measure_duration:
                measure_buffer.add(rest_fill())

            note = map_ir_event_to_note(ir_event, current_divisions)
            inject_provenance(note, ir_event.provenance, config.profile)
            measure_buffer.add(note)

            if measure_buffer.is_full():
                part.measures.add(measure_buffer.flush())

        doc.parts.add(part)

    return serialize_xml(doc)
```

### 9.2 Simultaneous Chord Stacking

When IR contains simultaneous notes at same onset (true chord or voice collision):

1. First note: full `<note>` element
2. Subsequent notes: `<note>` with `<chord/>` as first child (before `<pitch>`)

Voice collision policy: if two IR voices share a part and collide, split to separate `<voice>` numbers within same `<part>`.

### 9.3 MusicXML-to-AST Import (Outline)

```
function import_musicxml(xml: String) -> Composition:
    doc = parse_and_validate(xml)
    assert doc.root == score-partwise

    composition = new Composition()
    composition.metadata = extract_metadata(doc.identification)

    for each part in doc.parts:
        voice = create_voice_from_part(part)
        current_measure = None

        for each measure in part.measures:
            m = create_measure(measure.number)
            extract_attributes(measure, m)

            for each note_event in measure.notes:
                event = map_note_to_ast_event(note_event)
                event.provenance = extract_provenance(note_event) ?? Provenance.manual_import()
                m.add_event(voice, event)

            for each harmony in measure.harmonies:
                m.add_chord_symbol(map_harmony(harmony))

            for each direction in measure.directions:
                map_direction_to_marker(direction, composition)

            voice.add_measure(m)

        composition.add_voice(voice)

    rebuild_section_tree_from_markers(composition)
    validate_ast(composition)
    return composition
```

### 9.4 Measure Alignment Validation

Import must verify all parts have equal measure counts (partwise invariant). Mismatch → error `E_MXML_MEASURE_MISMATCH` with part IDs and counts.

---

## 10. Interfaces

### 10.1 Rust Public API

```rust
/// Export configuration
pub struct MusicXmlExportConfig {
    pub profile: MusicXmlProfile,       // Interchange | Publish | Preview | Full
    pub divisions_per_quarter: u32,     // default 480
    pub part_mapping: PartMappingMode,  // OnePartPerVoice | VoiceGroupsToStaves
    pub include_provenance: bool,
    pub pretty_print: bool,
}

pub enum MusicXmlProfile {
    Interchange,
    Publish,
    Preview,
    Full,
}

/// Primary export entry point
pub fn export_musicxml(
    ast: &Composition,
    ir: &MusicIR,
    config: &MusicXmlExportConfig,
) -> Result<String, ExportError>;

/// Import entry point
pub fn import_musicxml(xml: &str) -> Result<Composition, ImportError>;

/// Validate without full import
pub fn validate_musicxml(xml: &str) -> Result<ValidationReport, ImportError>;
```

### 10.2 Tauri IPC Commands

```typescript
// Frontend invoke signatures
export_composition_musicxml(composition_id: string, config: MusicXmlExportConfig): Promise<string>
import_musicxml_file(path: string): Promise<CompositionHandle>
validate_musicxml_content(xml: string): Promise<ValidationReport>
```

### 10.3 ExportPlugin Hook

```rust
trait MusicXmlExportPlugin {
    fn post_process_document(&self, doc: &mut MusicXmlDocument) -> Result<(), PluginError>;
    fn custom_directions(&self, section: &Section) -> Vec<Direction>;
}
```

---

## 11. Parameter Mappings

### 11.1 User Export Parameters

| User Parameter | Export Config Field | Effect |
|----------------|---------------------|--------|
| `export.format` | *(implicit)* | Selects MusicXML exporter |
| `export.provenance` | `include_provenance` | Tier B attributes on/off |
| `export.engraving_quality` | `profile` | Publish adds layout hints |
| `voice.count` | part_list size | Number of `<score-part>` entries |
| `style.orchestration_preset` | instrument names, MIDI program | `<score-instrument>`, `<midi-program>` |
| `mode.key` | first measure `<key>` | Fifths + mode |
| `rhythm.time_signature` | `<time>` | Beats + beat-type |

### 11.2 AST Node → MusicXML Element Mapping Table (Normative)

#### Root and Metadata

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Composition` | `<score-partwise>` | Root element |
| `Composition.metadata.title` | `<work><work-title>` | |
| `Composition.metadata.composer` | `<credit><credit-type>composer</credit-type>` | |
| `Composition.metadata.created_at` | `<encoding><encoding-date>` | ISO 8601 |
| `Composition.metadata.parameters` | `<identification><miscellaneous>` + `aurora:params` | JSON in miscellaneous-field |
| `Composition.metadata.provenance_root` | `<identification><miscellaneous>` | Generation run ID |
| `GlobalAttributes` | `<defaults>` + first `<attributes>` | Divisions, key, time, clef |

#### Form Structure

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Movement` | `<part>` grouping or `<credit>` page break | Single-movement: implicit; multi: `new-page="yes"` |
| `Section.role` | `<direction><rehearsal>` or `<direction><words>` | `aurora:section-role` attribute |
| `Section.label` | `<direction><rehearsal>` | e.g. "A", "B", "Chorus" |
| `Phrase` | `<direction><words>` (italic) | Optional phrase marks via `<slur>` |
| `SectionMarker` | `<barline location="left">` + `<direction>` | Section boundary |

#### Measure and Attributes

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Measure` | `<measure number="">` | 1-based index |
| `Measure.time_signature` | `<attributes><time>` | Mid-score changes |
| `Measure.key_signature` | `<attributes><key>` | `<fifths>`, `<mode>` |
| `Measure.clef` | `<attributes><clef>` | Per-staff clef changes |
| `Measure.divisions` | `<attributes><divisions>` | Export config default |

#### Voice / Part

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Voice` | `<score-part>` + `<part id="">` | One part per voice (default) |
| `Voice.name` | `<part-name>` | |
| `Voice.instrument` | `<score-instrument>`, `<midi-instrument>` | GM program |
| `VoiceGroup` | Multiple `<score-part>` linked via `aurora:voice-group` | Orchestration preset |
| `Voice.midi_channel` | `<midi-channel>` | 1–16; drums → channel 10 |

#### Events — Note

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Event::Note.pitch` | `<pitch><step><octave>` | + `<alter>` for accidentals |
| `Event::Note.duration` | `<duration>`, `<type>`, `<dot>` | Divisions + visual type |
| `Event::Note.velocity` | `<note><dynamics>` or `<sound dynamics="">` | Playback |
| `Event::Note.staccato` | `<notations><articulations><staccato/>` | |
| `Event::Note.tenuto` | `<articulations><tenuto/>` | |
| `Event::Note.accent` | `<articulations><accent/>` | |
| `Event::Note.fermata` | `<fermata>` | |
| `Event::Note.tuplet` | `<time-modification>`, `<tuplet>` | |
| `Event::Note.tie_start/end` | `<tie type="start/stop">` | |
| `Event::Note.slur` | `<slur type="start/stop">` | |
| `Event::Note.grace` | `<grace>`, `<grace slash="">` | |
| `Event::Note.trill` | `<ornaments><trill-mark/>` | |
| `Event::Note.mordent` | `<ornaments><mordent/>` | |
| `Event::Note.provenance` | `aurora:provenance` attribute | Tier B |
| `Event::Note.id` | `aurora:id` attribute | UUID |

#### Events — Rest

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Event::Rest.duration` | `<rest/>`, `<duration>`, `<type>` | |
| `Event::Rest.measure_rest` | `<rest measure="yes"/>` | Full bar rest |
| `Event::Rest.provenance` | `aurora:provenance` | Tier B |

#### Events — Chord (Harmony)

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Event::Chord.symbol` | `<harmony><root>`, `<kind>`, `<bass>` | MusicXML chord symbols |
| `Event::Chord.roman_numeral` | `<harmony><function>` | Classical analysis |
| `Event::Chord.offset` | `<harmony>` placement via `<offset>` | Off-beat harmony |
| `Event::Chord.voicing` | Multiple `<note><chord/>` stack | Separate from symbol |

#### Events — Marker

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Marker::TempoChange` | `<direction><metronome>` or `<sound tempo="">` | |
| `Marker::DynamicChange` | `<direction><dynamics>` | |
| `Marker::RehearsalMark` | `<direction><rehearsal>` | |
| `Marker::TextAnnotation` | `<direction><words>` | |

#### Drums (Unpitched)

| AST Node / Field | MusicXML Element | Notes |
|------------------|------------------|-------|
| `Event::Note (drum)` | `<note><unpitched>` | `<display-step>`, `<display-octave>` |
| `Voice[drums].instrument` | `<score-instrument id="P10-I1">` | `<instrument-sound>drum</instrument-sound>` |
| GM drum pitch | `<unpitched><display-step>` mapping | See §11.3 |

### 11.3 GM Drum Map (Unpitched Display)

| AST Drum ID | MIDI Note | MusicXML display-step/octave |
|-------------|-----------|------------------------------|
| kick | 36 | F, 4 (convention) |
| snare | 38 | C, 5 |
| closed_hihat | 42 | G, 5 |
| open_hihat | 46 | A, 5 |
| crash | 49 | C, 6 |
| ride | 51 | D, 6 |

Channel 10 in `<midi-instrument>`; `<unpitched>` for each note.

---

## 12. Explainability Model

### 12.1 Provenance in MusicXML

Every generated `Event` exported with `profile = interchange` carries `aurora:provenance`. The UI Inspector reads provenance from AST; when user imports external MusicXML, provenance displays as:

```text
Origin: External import
Source: MuseScore 4.x
Provenance: unavailable (Tier B stripped)
```

### 12.2 Traceability Chain

```text
User clicks note in score view
  → AST Event.id
  → aurora:id in MusicXML (if interchange file loaded)
  → Provenance chain: stage → rule_ids → eval_score → parent_id
  → UI renders "Generated by Melody stage, rules VL-001 + HR-042, score 0.847"
```

### 12.3 Export Audit Log

Export job records:

```json
{
  "format": "musicxml",
  "profile": "interchange",
  "tier_a_events": 1247,
  "tier_b_events": 1247,
  "validation_warnings": [],
  "output_sha256": "..."
}
```

---

## 13. Future Expansion

| Phase | Feature |
|-------|---------|
| v0.2 | `.mxl` ZIP packaging |
| v0.2 | Timewise import (normalize to partwise) |
| v0.3 | Lyrics `<lyric>` from vocal plugin |
| v0.3 | Tablature `<frame>` for guitar plugin |
| v0.4 | MusicXML 4.1+ schema updates |
| v0.4 | Compressed provenance (external ref table) |
| v0.5 | MEI export plugin (scholarly) |

---

## 14. Open Questions

1. **Multi-staff single part:** Should piano L/R be one `<part>` with two `<staff>` or two Aurora voices? Default: one part, two staves for piano preset.
2. **Microtonal:** `<pitch><alter>` quarter-tones — required for Phase 1 or defer?
3. **MXL mandatory?** Some tools prefer `.mxl`; prioritize in v0.1 or v0.2?
4. **Validation strictness:** Fail import on schema warnings or best-effort repair?
5. **Conflict with pending AST schema:** Reconcile when `ast.md` publishes — Architecture Agent review required.

---

## 15. References

- W3C MusicXML 4.0 Specification (2021): https://www.w3.org/2021/06/musicxml40/
- Recordare MusicXML DTDs and XSDs
- Music21 MusicXML converter source (MIT)
- MuseScore 4 MusicXML compatibility notes
- Verovio supported MusicXML subset documentation
- [ACAS v0.1 §9](../00-overview/acas-v0.1.md)
- [ADR-004 MusicXML Primary Export](../../decisions/ADR-004-musicxml-primary-export.md)
- [Export Research Notes](../../research/export-research-notes.md)
- [MIDI Export Specification](midi.md)
- [PDF Rendering Specification](pdf.md)
- [Terminology — Music AST](../00-overview/terminology.md)

---

## Appendix A: Complete Mapping Reference (Extended)

### A.1 GlobalAttributes Field Mapping

| AST `GlobalAttributes` Field | MusicXML Location | Import Reverse Map |
|------------------------------|-------------------|-------------------|
| `title` | `<work><work-title>` | `Composition.metadata.title` |
| `composer` | `<credit type="composer">` | `Composition.metadata.composer` |
| `copyright` | `<rights>` | `Composition.metadata.copyright` |
| `software` | `<encoding><software>` | `Composition.metadata.generator` |
| `ticks_per_quarter` | *(internal only)* | Derived from `<divisions>` |
| `tempo_map[0].bpm` | `<sound tempo="">` or `<metronome>` | `GlobalAttributes.tempo_map` |
| `key.fifths` | `<key><fifths>` | `GlobalAttributes.key` |
| `key.mode` | `<key><mode>` | `GlobalAttributes.key.mode` |
| `time.beats` | `<time><beats>` | `GlobalAttributes.time_signature` |
| `time.beat_type` | `<time><beat-type>` | `GlobalAttributes.time_signature` |
| `pickup_measure` | `<measure implicit="yes">` | `Movement.pickup` |

### A.2 Ornament and Articulation Mapping

| AST Field | MusicXML Element | Export Priority |
|-----------|------------------|-----------------|
| `staccato` | `<articulations><staccato/>` | P1 |
| `staccatissimo` | `<staccato/>` (no distinct — use `<other-articulation>`) | P2 |
| `tenuto` | `<tenuto/>` | P1 |
| `accent` | `<accent/>` | P1 |
| `marcato` | `<strong-accent/>` | P1 |
| `fermata` | `<fermata type="upright"/>` | P1 |
| `trill` | `<ornaments><trill-mark/>` + optional `<wavy-line/>` | P1 |
| `turn` | `<turn/>` | P2 |
| `mordent` | `<mordent/>` | P1 |
| `inverted_mordent` | `<inverted-mordent/>` | P2 |
| `tremolo` | `<ornaments><tremolo>` | P2 |
| `arpeggiate` | `<arpeggiate/>` | P2 |
| `breath_mark` | `<articulations><breath-mark/>` | P3 |
| `caesura` | `<articulations><caesura/>` | P3 |

Priority P1 = Tier A required; P2 = Tier A best-effort; P3 = Tier C.

### A.3 Harmony (Chord Symbol) Mapping

| AST `Event::Chord` Field | MusicXML `<harmony>` Child | Example |
|--------------------------|---------------------------|---------|
| `root_step` | `<root><root-step>` | C |
| `root_alter` | `<root><root-alter>` | -1 for Cb |
| `kind` | `<kind>` text attribute | major, minor, dominant, etc. |
| `bass_step` | `<bass><bass-step>` | E in C/E |
| `degrees[]` | `<degree>` | add9, alter11 |
| `symbol_string` | *(derived)* | Display via kind + degrees |
| `offset_ticks` | `<offset>` | Off-beat placement |

### A.4 Direction and Marker Mapping

| AST `Marker` Variant | MusicXML | Placement |
|---------------------|----------|-----------|
| `TempoChange { bpm }` | `<direction><direction-type><metronome>` | Above first staff |
| `DynamicChange { dynamic }` | `<direction><direction-type><dynamics>` | Below staff |
| `RehearsalMark { label }` | `<direction><rehearsal>` | Above staff |
| `TextAnnotation { text }` | `<direction><words>` | Configurable |
| `SectionBoundary { role }` | `<direction><words>` + `aurora:section-role` | Above staff |
| `Coda / Segno` | `<direction><segno>` / `<coda>` | Standard symbols |

---

## Appendix B: XML Examples

### B.1 Minimal Single-Voice Export

```xml
<?xml version="1.0" encoding="UTF-8"?>
<score-partwise version="4.0"
    xmlns:aurora="http://aurora-composer.dev/ns/v1">
  <identification>
    <encoding>
      <software>Aurora Composer 0.1</software>
      <encoding-date>2026-07-04</encoding-date>
    </encoding>
    <miscellaneous>
      <miscellaneous-field name="aurora-namespace">http://aurora-composer.dev/ns/v1</miscellaneous-field>
    </miscellaneous>
  </identification>
  <defaults>
    <scaling><millimeters>7</millimeters><tenths>40</tenths></scaling>
  </defaults>
  <part-list>
    <score-part id="P1">
      <part-name>Melody</part-name>
      <score-instrument id="P1-I1">
        <instrument-name>Piano</instrument-name>
      </score-instrument>
      <midi-instrument id="P1-I1">
        <midi-channel>1</midi-channel>
        <midi-program>1</midi-program>
      </midi-instrument>
    </score-part>
  </part-list>
  <part id="P1">
    <measure number="1">
      <attributes>
        <divisions>480</divisions>
        <key><fifths>0</fifths><mode>major</mode></key>
        <time><beats>4</beats><beat-type>4</beat-type></time>
        <clef><sign>G</sign><line>2</line></clef>
      </attributes>
      <note aurora:id="550e8400-e29b-41d4-a716-446655440001"
            aurora:provenance='{"v":1,"origin":"generation","stage":"melody","rule_ids":["ML-001"],"eval_score":0.91,"search_step":42}'>
        <pitch><step>C</step><octave>4</octave></pitch>
        <duration>480</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>E</step><octave>4</octave></pitch>
        <duration>480</duration>
        <type>quarter</type>
      </note>
      <note>
        <pitch><step>G</step><octave>4</octave></pitch>
        <duration>960</duration>
        <type>half</type>
      </note>
    </measure>
  </part>
</score-partwise>
```

### B.2 Multi-Voice Chord Stack

```xml
<!-- Voice 1 and Voice 2 sharing harmonic rhythm in measure 2 -->
<measure number="2">
  <note><!-- Voice 1, beat 1 -->
    <pitch><step>C</step><octave>4</octave></pitch>
    <duration>480</duration>
    <voice>1</voice>
    <type>quarter</type>
  </note>
  <note><!-- Voice 2 simultaneous chord tone -->
    <chord/>
    <pitch><step>E</step><octave>4</octave></pitch>
    <duration>480</duration>
    <voice>2</voice>
    <type>quarter</type>
  </note>
  <harmony>
    <root><root-step>C</root-step></root>
    <kind>major</kind>
  </harmony>
</measure>
```

### B.3 Drum Unpitched Note

```xml
<part id="P5">
  <measure number="1">
    <attributes>
      <clef><sign>percussion</sign></clef>
    </attributes>
    <note>
      <unpitched>
        <display-step>F</display-step>
        <display-octave>4</display-octave>
      </unpitched>
      <duration>240</duration>
      <instrument id="P5-I1"/>
      <voice>1</voice>
      <type>eighth</type>
    </note>
  </measure>
</part>
```

---

## Appendix C: Round-Trip Test Catalog

### C.1 Tier A Test Cases (Required Pass)

| Test ID | Description | Assert |
|---------|-------------|--------|
| RT-A01 | C major scale quarter notes | Pitch sequence equal |
| RT-A02 | Accidentals (F#, Bb) | `<alter>` round-trip |
| RT-A03 | Dotted half note | Duration + type |
| RT-A04 | Triplet eighths | `<time-modification>` + `<tuplet>` |
| RT-A05 | Tie across barline | `<tie start/stop>` |
| RT-A06 | 4/4 → 3/4 mid-score | `<attributes><time>` |
| RT-A07 | Key change G → D | `<key>` fifths |
| RT-A08 | Multi-part 4 voices | Part count + measure alignment |
| RT-A09 | Chord symbol Am7 | `<harmony>` fields |
| RT-A10 | Piano grand staff (2 staves) | Staff numbers |
| RT-A11 | Drum kit channel 10 | `<unpitched>` + MIDI channel |
| RT-A12 | Repeat barlines | `<repeat>` + `<barline>` |
| RT-A13 | Dynamics mf → ff | `<dynamics>` |
| RT-A14 | Tempo change 120 → 96 | `<sound tempo>` |
| RT-A15 | Grace note before beat | `<grace/>` |
| RT-A16 | Rest + measure rest | `<rest measure="yes"/>` |
| RT-A17 | Simultaneous chord tones | `<chord/>` stacking |
| RT-A18 | Pickup (anacrusis) measure | `implicit="yes"` |

### C.2 Tier B Test Cases (Aurora Interchange)

| Test ID | Description | Assert |
|---------|-------------|--------|
| RT-B01 | Provenance on generated note | JSON parse equal |
| RT-B02 | Event UUID stable | `aurora:id` preserved |
| RT-B03 | Theme reference | `aurora:theme-ref` |
| RT-B04 | Section role marker | `aurora:section-role` |
| RT-B05 | Generation parameters in metadata | `<miscellaneous>` JSON |
| RT-B06 | Voice group attribute | `aurora:voice-group` |

### C.3 Third-Party Import Tests

| Test ID | Source | Expected |
|---------|--------|----------|
| IMP-01 | MuseScore 4 export sample | Tier A pass; Tier B stub |
| IMP-02 | Music21 corpus fragment | Tier A pass |
| IMP-03 | Verovio test suite file | Tier A pass |
| IMP-04 | Malformed `<duration>` | Error E_MXML_INVALID_DURATION |

---

## Appendix D: Validation Rules

### D.1 Export Validation (Strict)

Before writing file, `MusicXmlValidator` checks:

1. All parts have equal measure count
2. Sum of note durations per voice per measure equals measure duration (within 1 division tolerance for rounding)
3. `<divisions>` consistent within part unless explicit change
4. `aurora:provenance` JSON parses against schema v1
5. No overlapping `<chord/>` stacks with conflicting voices unless same onset
6. MIDI channels 1–16; channel 10 only on percussion parts
7. `<score-part>` IDs referenced by `<part id="">` exist in `<part-list>`

### D.2 Import Validation (Liberal)

Import accepts files with warnings:

| Condition | Severity | Action |
|-----------|----------|--------|
| Unknown XML elements | Warning | Skip element |
| Missing `<divisions>` | Error | Abort import |
| Measure count mismatch | Error | Abort import |
| Invalid `aurora:provenance` JSON | Warning | Stub provenance |
| MusicXML 3.1 file | Warning | Upgrade mapping to 4.0 model |
| Timewise document | Info | Convert to partwise |

### D.3 Fidelity Guarantee Summary

| Guarantee | Scope | Profile |
|-----------|-------|---------|
| **G1 — Pitch/Rhythm Integrity** | All Tier A elements survive Aurora ↔ MusicXML ↔ Aurora | `interchange` |
| **G2 — Provenance Integrity** | Tier B metadata survives Aurora round-trip | `interchange` |
| **G3 — Interoperability** | Tier A survives export → MuseScore 4 → import | `publish` |
| **G4 — Preview Subset** | Verovio renders without error | `preview` |
| **G5 — No Silent Data Loss** | Export logs warnings for unmappable AST nodes | all profiles |

Unmappable nodes (e.g., internal search state) are never silently dropped — logged in `ExportReport.warnings[]`.

---

## Appendix E: Partwise vs Timewise — Decision Record

**Decision:** Partwise canonical (see ADR-004).

**Rationale summary:**

1. Aurora `Voice` maps 1:1 to `<part>` — natural serialization order
2. Export pipeline processes one voice at a time (matches IR voice streams)
3. MuseScore, Finale, Sibelius default to partwise export
4. Verovio expects partwise in documentation examples
5. Timewise import supported via normalization algorithm:

```
function timewise_to_partwise(doc: TimewiseDocument) -> PartwiseDocument:
    parts = init_parts(doc.part_count)
    for measure in doc.measures:
        for part_data in measure.parts:
            parts[part_data.id].measures.add(part_data.content)
    return PartwiseDocument(parts)
```

Measure count inferred from timewise `<measure>` sequence; all parts must appear in each measure (empty `<measure/>` allowed).

---

## Appendix F: Conflict Notes (Pending AST Spec)

The following mappings are **provisional** until `docs/02-music-model/ast.md` publishes:

| Topic | Provisional Assumption |
|-------|------------------------|
| Event ID type | UUID v4 string |
| `Voice` vs `Part` | One MusicXML part per Aurora voice |
| `Movement` in single-movement pop songs | Omitted; implicit single movement |
| `Phrase` boundary | `<direction><words>` italic; no dedicated element |
| Microtonal `Event::Note.pitch.cents` | Export as `<alter>` +24 max; else warning |

Architecture Agent must reconcile upon AST spec freeze.

---

*End of MusicXML Export/Import Specification v0.1*
