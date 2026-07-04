# Terminology & Glossary

**Document:** Aurora Composer — Terminology  
**Version:** 0.1  
**Status:** Draft  
**Authority:** This glossary is normative. All specifications and code must use these terms consistently.

---

## Core Concepts

### Music AST (Abstract Syntax Tree)

The canonical internal representation of a musical composition. A typed tree of nodes (Composition, Section, Measure, Voice, Event, etc.) that is the **only** data structure modules may read or write. See `docs/02-music-model/ast.md`.

### Music IR (Intermediate Representation)

A flattened, export-optimized projection of the AST used by exporters and playback. Derived from AST; never edited directly by generation modules. See `docs/02-music-model/ir.md`.

### Composition Engine

The Rust core that orchestrates the generation pipeline, rule engine, search, and AST mutations. Runs inside Tauri backend.

### Rule Engine

The subsystem that evaluates rules against AST states, producing scores and constraint violations. See `docs/05-rule-engine/`.

### Constraint

A boolean predicate on AST state. **Hard constraints** must be satisfied (search prunes violations). **Soft constraints** contribute to scoring (violations incur penalties).

### Rule

A named, documented condition that contributes to scoring or constraint checking. Rules are defined in the Rule DSL. Each rule has an ID, category, description, and parameter bindings.

### Score (Evaluation Score)

A numeric value assigned to a candidate AST state or event by the rule engine. Distinct from musical "score" (sheet music). Notation: `eval_score` when ambiguous.

### Search

The process of exploring candidate AST states under constraints, using algorithms such as beam search, A*, or dynamic programming.

### Provenance

Metadata attached to every generated event recording: reason, rule ID, eval_score, search step, and parent state reference.

### Parameter

A user-facing control mapped to internal algorithm weights, probabilities, and constraint thresholds. See Parameter System in ACAS.

### Plugin

An externally loaded module implementing a documented interface to extend styles, algorithms, or export formats.

---

## Structural Terms

### Composition

Root AST node. Contains metadata (title, composer, parameters used) and one or more Movements.

### Movement

A major division of a composition (e.g., first movement of a sonata). Contains Sections.

### Section

A structural unit with a defined role: exposition, development, recapitulation, verse, chorus, bridge, intro, outro, coda, etc.

### Phrase

A musical sentence — typically 2–8 measures — with coherent melodic/harmonic content and cadential boundary.

### Measure (Bar)

A time-bounded container aligned to the time signature. Contains Events for each Voice.

### Voice (Part)

A single melodic/harmonic line within the texture. Not necessarily "vocal" — includes melody, alto, tenor, bass, and percussion voices.

### Voice Group

A logical grouping of voices (e.g., "string section", "rhythm section").

### Event

An atomic timed element: Note, Chord, Rest, Marker, or Automation point.

### Marker

An annotation on the timeline: section boundary, rehearsal mark, tempo change, dynamic mark, articulation region.

---

## Generative Terms

### Generation Pipeline

The ordered sequence of engine stages from user parameters to validated AST. See `docs/01-architecture/pipeline.md`.

### Style Resolver

Pipeline stage that maps genre/style presets to parameter bundles and plugin selections.

### Emotion Resolver

Pipeline stage that maps emotion labels (valence, arousal, tension) to harmonic, rhythmic, and melodic weight adjustments.

### Structure Planning

Pipeline stage that determines sectional form, measure count, key areas, and tempo map before note-level generation.

### Theme Planning

Pipeline stage that allocates themes to sections and plans motif development strategy.

### Harmony Skeleton

Chord progression framework (harmonic rhythm + chord symbols) before voicing and melody.

### Rhythm Skeleton

Metric framework (patterns, subdivisions, accent structure) before note-level rhythm filling.

### Skeleton

Any coarse-grained plan (harmony, rhythm, form) filled in by subsequent pipeline stages.

### Motif

A short, identifiable musical cell (typically 1–4 beats) used as building block for themes.

### Theme

An expanded motif or phrase (typically 1–4 measures) assigned to a section. Themes may be transformed (sequence, inversion, augmentation).

### Counterpoint

The simultaneous combination of independent melodic lines following voice-leading rules.

### Decoration

Ornamental notes added after primary melody/harmony: trills, mordents, passing tones, appoggiaturas.

### Repair

Post-generation stage that fixes soft-constraint violations, voice-leading issues, or range overflows without full regeneration.

### Validation

Final check that all hard constraints are satisfied and export prerequisites are met.

---

## Theory Terms

### Key / Mode

Tonal center and scale type (major, natural minor, harmonic minor, dorian, etc.).

### Scale

Ordered pitch collection derived from key/mode.

### Degree

Scale step number (I–VII) used for harmonic analysis.

### Chord Symbol

Human-readable chord name (e.g., `Am7`, `G7/B`, `Dbmaj7#11`).

### Voicing

Specific pitch assignment of chord tones across voices.

### Voice Leading

Rules governing motion between consecutive chords (common tones, stepwise preference, parallel interval avoidance).

### Cadence

Harmonic formula marking phrase/section ending (authentic, plagal, half, deceptive).

### Tension

Harmonic or rhythmic dissonance level, parameterized and tracked over form.

### Texture

Homophonic (melody + accompaniment), polyphonic (independent lines), or mixed.

---

## Technology Terms

### ACAS

Aurora Composer Architecture Specification — the master specification document set.

### Rule DSL

Domain-specific language for defining rules and constraints. See `docs/05-rule-engine/rule-dsl.md`.

### Tauri

Rust-based framework for building desktop apps with web frontends. Aurora Composer's shell.

### Exporter

Module projecting AST/IR to external formats (MIDI, ABC, MusicXML, PDF).

### Design Freeze

The milestone at which architecture specifications are considered stable and implementation may begin.

### ADR

Architecture Decision Record — documented in `decisions/`.

---

## Abbreviations

| Abbr | Meaning |
|------|---------|
| ACAS | Aurora Composer Architecture Specification |
| AST | Abstract Syntax Tree |
| IR | Intermediate Representation |
| DSL | Domain-Specific Language |
| ADR | Architecture Decision Record |
| MIDI | Musical Instrument Digital Interface |
| XML | MusicXML |
| UI | User Interface |
| ML | Machine Learning |
| DP | Dynamic Programming |
| PCFG | Probabilistic Context-Free Grammar |
| BPM | Beats Per Minute |

---

## References

- [ACAS v0.1](acas-v0.1.md)
- [Music AST Specification](../02-music-model/ast.md)
