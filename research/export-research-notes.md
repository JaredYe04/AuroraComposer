# Export Format Research Notes

**Version:** 0.1  
**Status:** Draft (raw research)  
**Agent:** Export Format Research Agent  
**Date:** 2026-07-04

---

## Purpose

Raw background research supporting `docs/06-export/` specifications. Not normative — see module specs for authoritative design.

---

## 1. Standards Survey

### MusicXML

- **Current version:** MusicXML 4.0 (W3C community standard, 2021)
- **Schema locations:** `http://www.musicxml.org/xsd/partwise` and `timewise`
- **Key references:**
  - [MusicXML 4.0 Tutorial](https://www.w3.org/2021/06/musicxml40/tutorial/introduction/)
  - [MusicXML 4.0 Music Data Representation](https://www.w3.org/2021/06/musicxml40/musicdatatypes/)
  - W3C Music Notation Community Group specifications
- **Partwise vs timewise:**
  - Partwise: `<part>` elements each contain sequential `<measure>` — natural for score-centric editing
  - Timewise: `<measure>` elements contain all `<part>` data — natural for measure-at-a-time editing
  - Industry practice: MuseScore, Finale, Sibelius export partwise by default
- **Extension mechanism:** `other-*` attributes, `<other-notation>`, `<other-direction>`, custom XML namespaces on elements (with validation trade-offs)

### Standard MIDI Files (SMF)

- **Spec:** MIDI 1.0 Specification + RP-016 SMF
- **Formats:** Type 0 (single track), Type 1 (multi-track), Type 2 (multi-sequence — rare)
- **Tick resolution:** TPQN (ticks per quarter note), typically 480 or 960
- **Channel 10:** General MIDI percussion (drum kit map)
- **Meta events:** Set Tempo (FF 51 03), Time Signature (FF 58 04), Key Signature (FF 59 02)
- **Rust libraries surveyed:** `midly`, `rimd` (archived but reference), custom writer

### ABC Notation

- **Spec:** ABC 2.1 (2011) + abc2xml extensions
- **Voice syntax:** `V:` header for multi-voice; `%%score` for grouping
- **Limitations:** ~4 voices practical; chord symbols via `w:`; no native provenance
- **Tools:** abcm2ps, abc2midi, abcjs (browser), EasyABC
- **Use case:** Folk, trad, quick sharing; not orchestral scores

### PDF Rendering

| Tool | Approach | Pros | Cons |
|------|----------|------|------|
| **Verovio WASM** | MusicXML → SVG → PDF | In-browser, no system deps, fast for preview | Engraving quality good but not MuseScore-level; WASM bundle ~2–4 MB |
| **MuseScore CLI** | MusicXML → PDF via headless MuseScore 4 | High-quality engraving | Requires MuseScore install; headless setup varies by OS |
| **LilyPond** | MusicXML → LilyPond → PDF | Excellent quality | Extra conversion step; slower |
| **abc2ps/abcm2ps** | ABC → PostScript/PDF | Lightweight | Only for ABC export path |

---

## 2. Prior Art: Music21 Export Patterns

Music21 (Python) provides reference mappings:

- `Score` → MusicXML `<score-partwise>`
- `Part` → `<part>` with `<score-part>` definition
- `Measure` → `<measure>`
- `Note` → `<note>` with `<pitch>` or `<unpitched>` for drums
- `Chord` → multiple `<note>` elements with `<chord/>` child on subsequent notes
- `Rest` → `<rest>`
- `Key`, `TimeSignature`, `Clef` → `<attributes>`

Aurora differs from Music21:

- Aurora has **Provenance** on every event (Music21 has `editorial` but not generation trace)
- Aurora has **Section/Phrase** hierarchy above Measure (Music21 uses `Repeat`, `RehearsalMark`)
- Aurora **Voice** maps to `<part>` not `<staff>` within single part (configurable)

---

## 3. Verovio Integration Notes

- **Repository:** https://github.com/rism-digital/verovio
- **WASM build:** `verovio-toolkit-wasm` npm package
- **API:** `loadData(xmlString)`, `renderToSVG(page)`, `getPageCount()`
- **Options:** `scale`, `pageWidth`, `pageHeight`, `adjustPageHeight`
- **Limitations:** Some MusicXML 4.0 elements unsupported (pedal lines, complex cross-staff beaming); test against Aurora output subset
- **PDF path:** SVG → canvas → jsPDF or browser print-to-PDF; or server-side via headless Chromium

---

## 4. MuseScore CLI Notes

```bash
# MuseScore 4 headless (typical)
mscore -o output.pdf input.musicxml
# or
MuseScore4.exe -o output.pdf input.musicxml
```

- Tauri backend can invoke via `Command` with user-configured path
- Fallback when Verovio quality insufficient (final print export)
- Job queue: async export with progress callback

---

## 5. Tone.js / Web Audio Playback Notes

- **@tonejs/midi** — parse MIDI ArrayBuffer, schedule notes on Tone.Synth or Sampler
- **soundfont-player** — GM soundfonts in browser (~2–20 MB per font)
- **FluidSynth** — system-level; not default for Tauri (dependency weight)
- **Latency target:** <50 ms for preview; acceptable for composition tool
- **Architecture:** IR events → Web Audio scheduling graph; not full MIDI file round-trip for preview

---

## 6. Fidelity Loss Matrix (Preliminary)

| AST Concept | MusicXML | MIDI | ABC | PDF |
|-------------|----------|------|-----|-----|
| Pitch + rhythm | ✓ | ✓ | ✓ | ✓ |
| Multi-voice polyphony | ✓ | ✓ (tracks) | △ (≤4 voices) | ✓ |
| Chord symbols | ✓ harmony | ✗ | △ w: field | ✓ |
| Form (Section/Phrase) | △ markers | ✗ | ✗ | △ |
| Provenance | ✓ aurora:* | ✗ | ✗ | ✗ |
| Ornaments | ✓ | △ (pitch bend) | △ | ✓ |
| Dynamics | ✓ | △ (velocity) | △ | ✓ |
| Tempo map | ✓ | ✓ | △ | ✓ |
| Drum kit mapping | ✓ unpitched | ✓ ch.10 | △ | ✓ |

Legend: ✓ full, △ partial, ✗ lost

---

## 7. Open Research Items

1. Register `aurora-composer.dev` namespace formally or use `software` attribute pattern?
2. MusicXML `<credit>` for generation parameters — useful or clutter?
3. Optimal TPQN for MIDI export (480 vs 960) given Aurora's tuplet support
4. Verovio bundle size vs lazy-load strategy for Tauri frontend
5. GM soundfont licensing for bundled distribution (MusyngKite, FluidR3, etc.)

---

## References

- W3C MusicXML 4.0 Specification (2021)
- MIDI Manufacturers Association SMF specification
- ABC Notation Standard 2.1
- Music21 documentation (v9.x)
- Verovio documentation
- MuseScore 4 handbook — command line usage
- Tone.js and @tonejs/midi documentation
- [Deep Research Report](../deep-research-report.md)
