# ADR-004: MusicXML as Primary Export and Interchange Format

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)

## Context

Aurora Composer exports compositions to multiple formats: MusicXML, SMF MIDI, ABC notation, and PDF (rendered from notation). The engine maintains a rich Music AST with hierarchical form structure, multi-voice polyphony, chord symbols, ornaments, dynamics, and full provenance metadata.

External tools (MuseScore, Dorico, Sibelius, Verovio, Music21) and the Aurora PDF pipeline require a notation-centric interchange format. The deep research report and ACAS v0.1 both identify MusicXML as the natural primary format, but this choice was marked "ADR pending" and affects exporter priority, round-trip import design, and PDF rendering architecture.

Competing candidates:

- **MusicXML 4.0** — industry-standard symbolic notation interchange; supports multi-part scores, lyrics, directions, playback elements
- **SMF (Standard MIDI File)** — universal playback format; loses form hierarchy, chord symbols, and provenance
- **ABC notation** — compact folk/tradition format; limited polyphony and no provenance
- **Native Aurora JSON AST** — lossless for Aurora but unusable by external notation software

## Decision

Adopt **MusicXML 4.0 (partwise)** as the **primary export and interchange format** for Aurora Composer.

Specifically:

1. **Export priority:** MusicXML is generated first in the export pipeline; MIDI, ABC, and PDF are derived projections (PDF from MusicXML; MIDI/ABC from IR with documented fidelity loss).
2. **Document type:** Use **partwise** (`score-partwise`) as the canonical serialization; timewise import is supported but normalized to partwise on ingest.
3. **Provenance extension:** Register Aurora namespace `http://aurora-composer.dev/ns/v1` with custom attributes on `<note>`, `<rest>`, `<harmony>`, and `<score-part>` elements (see [musicxml.md](../docs/06-export/musicxml.md) §8.3).
4. **Round-trip contract:** Aurora-to-Aurora round-trip via MusicXML must satisfy **Fidelity Tier A + B** (see musicxml spec §6.2); import from third-party MusicXML satisfies Tier A only.
5. **Implementation language:** MusicXML export/import implemented in Rust core engine; PDF rendering delegated to Verovio WASM (primary) with MuseScore CLI fallback.

## Consequences

### Positive

- Maximum interoperability with notation software and Verovio/MuseScore PDF pipeline
- Single authoritative notation artifact for user file exchange
- Provenance can travel with the score via namespaced attributes without breaking standard parsers
- Aligns with Music21, academic tooling, and MAESTRO/Lakh validation workflows

### Negative

- MusicXML is verbose; large scores produce multi-MB files
- Import parser complexity is high (MusicXML 4.0 schema + vendor quirks)
- Some AST concepts (search provenance depth, theme slot references) have no standard MusicXML equivalent — require Aurora extension namespace
- Round-trip through non-Aurora tools strips Tier B metadata

### Neutral

- MIDI and ABC remain supported as secondary formats with explicit limitation documentation
- PDF is a render artifact, not a source-of-truth format

## Alternatives Considered

| Alternative | Rejected Because |
|-------------|------------------|
| **MIDI as primary** | Loses notation, form, chord symbols, voice separation semantics; unsuitable for sheet music exchange |
| **ABC as primary** | Limited to ~4 voices per staff group; no provenance; poor fit for classical/jazz multi-part scores |
| **Native JSON AST only** | Zero external tool compatibility; violates interchange goals |
| **Timewise MusicXML canonical** | Partwise maps naturally to Aurora's part/voice model; timewise complicates multi-voice export and is less common in toolchains |
| **MEI (Music Encoding Initiative)** | Richer for scholarly encoding but weaker commercial tool support vs MusicXML |
| **Dual primary (MusicXML + MIDI)** | Creates ambiguity about source of truth; ACAS mandates single AST with format projections |

## Related Documents

- [MusicXML Export Specification](../docs/06-export/musicxml.md)
- [MIDI Export Specification](../docs/06-export/midi.md)
- [PDF Rendering Specification](../docs/06-export/pdf.md)
- [ACAS v0.1 §9](../docs/00-overview/acas-v0.1.md)
- [Export Research Notes](../research/export-research-notes.md)
