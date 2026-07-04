# PDF Rendering Specification

**Version:** 0.1  
**Status:** Draft  
**Agent:** Export Format Research Agent (Notation/PDF)  
**Dependencies:** [musicxml.md](musicxml.md), [ADR-004](../../decisions/ADR-004-musicxml-primary-export.md), `docs/08-ui/vue.md` *(pending)*

---

## 1. Background

PDF export provides **print-ready engraved notation** for Aurora Composer users. PDF is a **render artifact**, not a source-of-truth format — it is always derived from MusicXML.

```text
Music AST  →  IR  →  MusicXML (publish profile)  →  PDF Renderer  →  .pdf
```

Two rendering backends:

1. **Verovio WASM (primary)** — in-browser, no external dependencies; used for preview and default PDF export
2. **MuseScore CLI (fallback)** — higher engraving quality when MuseScore 4 is installed; used for final print export

---

## 2. Existing Solutions

### 2.1 Verovio

- **Input:** MusicXML (partwise)
- **Output:** SVG per page
- **Deployment:** `verovio-toolkit-wasm` npm package (~2–4 MB WASM)
- **Quality:** Good for preview; less sophisticated spacing than MuseScore/Dorico
- **License:** LGPL / commercial dual license

### 2.2 MuseScore CLI

- **Command:** `MuseScore4 -o output.pdf input.musicxml`
- **Quality:** Professional-grade default engraving
- **Requirement:** User-installed MuseScore 4; path configurable in settings
- **Headless:** Supported on Windows, macOS, Linux with documented flags

### 2.3 LilyPond

- musicxml2ly conversion chain — excellent quality, slow, extra dependency. **Not recommended** for v0.1.

### 2.4 abcjs + jsPDF

- ABC path only; not applicable to full orchestral PDF from MusicXML primary pipeline.

---

## 3. Academic / Theoretical Foundation

Music engraving follows established conventions (Ross, Gould — *Behind Bars*):

- Spacing proportional to duration and metric weight
- Beam grouping by beat subdivision
- Collision avoidance between voices

PDF rendering is **presentation-layer** — it does not affect musical content. Aurora separates:

- **Tier A/B content** — in MusicXML
- **Tier C layout** — influenced by renderer; may differ between Verovio and MuseScore

---

## 4. Engineering Analysis

| Criterion | Verovio WASM | MuseScore CLI |
|-----------|--------------|---------------|
| Correctness | Good subset | Full MusicXML 4 |
| Setup | npm install | User path config |
| Performance | 200–800 ms / page | 1–5 s / page |
| Offline | Yes (WASM) | Yes (local binary) |
| Tauri integration | Frontend or embedded WASM | Backend Command spawn |
| Quality | Preview-grade | Print-grade |

---

## 5. Comparison of Approaches

| Approach | Verdict |
|----------|---------|
| IR → PDF direct | No mature library — rejected |
| MusicXML → Verovio → SVG → PDF | **Primary path** |
| MusicXML → MuseScore → PDF | **Fallback path** |
| MusicXML → Cloud API (Flat.io) | Network dependency — rejected per ACAS security |
| Client-side only | Works for preview; large scores may OOM |
| Server-side Tauri render | MuseScore CLI via Rust Command — recommended for final export |

---

## 6. Recommended Solution

### 6.1 Dual-Backend Strategy

| Use Case | Backend | Trigger |
|----------|---------|---------|
| In-app score preview | Verovio WASM | Auto on composition load |
| Quick PDF export | Verovio WASM → PDF | Default export button |
| High-quality PDF | MuseScore CLI | "Print quality" export option |
| Batch export | MuseScore CLI (Tauri job) | Background job queue |

### 6.2 MusicXML Input Profile

PDF pipeline consumes MusicXML with profile:

- **Preview:** `preview` — minimal `<print>`, no aurora provenance
- **Publish:** `publish` — layout hints, no provenance
- **Print (MuseScore):** `publish` or `full` without provenance bloat

Provenance attributes do not affect rendering; strip in `preview`/`publish` for smaller DOM.

---

## 7. Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                     PDF Rendering Pipeline                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    MusicXML      ┌─────────────────────────┐  │
│  │ Export Engine│ ───────────────► │   Renderer Router        │  │
│  │ (Rust)       │   publish profile │   (config.backend)       │  │
│  └──────────────┘                   └───────────┬─────────────┘  │
│                                                  │                 │
│                    ┌─────────────────────────────┼─────────────┐ │
│                    ▼                             ▼             │ │
│         ┌──────────────────┐         ┌──────────────────┐     │ │
│         │ Verovio WASM      │         │ MuseScore CLI     │     │ │
│         │ (Vue frontend or  │         │ (Tauri Command)   │     │ │
│         │  embedded worker) │         │                   │     │ │
│         └────────┬─────────┘         └────────┬─────────┘     │ │
│                  │ SVG[]                       │ PDF direct      │ │
│                  ▼                             │                 │ │
│         ┌──────────────────┐                  │                 │ │
│         │ SVG → PDF         │                  │                 │ │
│         │ (jsPDF / print)   │                  │                 │ │
│         └────────┬─────────┘                  │                 │ │
│                  └──────────────┬───────────────┘                 │ │
│                                 ▼                               │ │
│                            .pdf file                            │ │
└─────────────────────────────────────────────────────────────────┘
```

### 7.1 Frontend Integration (Verovio)

```text
Vue 3 ScorePreview Component
├── props: compositionId, pageIndex, scale
├── onMount: load Verovio WASM (lazy)
├── watch(compositionId): invoke export_musicxml(preview) → toolkit.loadData()
├── render: toolkit.renderToSVG(pageIndex) → v-html or DOMParser
└── emit: pageCount, renderComplete
```

### 7.2 Backend Integration (MuseScore)

```text
Tauri Command: export_pdf_musescore
├── Input: composition_id, output_path, quality_preset
├── Step 1: export_musicxml(ast, ir, publish) → temp .musicxml
├── Step 2: Command::new(musescore_path).args(["-o", pdf, musicxml])
├── Step 3: validate PDF exists; return path
└── Error: E_PDF_MUSESCORE_NOT_FOUND with install instructions
```

---

## 8. Data Structures

### 8.1 PdfExportConfig

```typescript
interface PdfExportConfig {
  backend: 'verovio' | 'musescore' | 'auto';  // auto: try musescore, fallback verovio
  pageSize: 'A4' | 'Letter';
  orientation: 'portrait' | 'landscape';
  scale: number;           // Verovio scale, default 100
  pageWidth: number;       // Verovio units, default 2100
  pageHeight: number;      // 0 = auto
  margins: MarginConfig;
  musescorePath?: string;  // user override
}
```

### 8.2 Verovio Toolkit Options

```javascript
const options = {
  scale: config.scale,
  pageWidth: config.pageWidth,
  pageHeight: config.pageHeight,
  adjustPageHeight: true,
  footer: 'none',
  header: 'none',
  mmOutput: true,  // for print sizing
};
toolkit.setOptions(options);
```

### 8.3 Render Job State

```rust
struct PdfRenderJob {
    job_id: Uuid,
    composition_id: Uuid,
    backend: PdfBackend,
    status: JobStatus,      // Queued | Rendering | Complete | Failed
    progress: f32,          // 0.0–1.0 (page index / page count)
    output_path: Option<PathBuf>,
    error: Option<PdfError>,
}
```

---

## 9. Algorithms

### 9.1 Verovio Render Loop

```
function render_pdf_verovio(musicxml: String, config: PdfExportConfig) -> Bytes:
    toolkit = await VerovioToolkit.create()
    toolkit.setOptions(config.to_verovio_options())
    toolkit.loadData(musicxml)

    page_count = toolkit.getPageCount()
    svg_pages = []

    for page in 1..page_count:
        svg_pages.add(toolkit.renderToSVG(page))

    pdf = svg_pages_to_pdf(svg_pages, config.pageSize, config.margins)
    return pdf
```

### 9.2 SVG to PDF Conversion

**Browser path (default):**

1. Parse SVG string to DOM
2. Draw each SVG to `<canvas>` via `canvg` or native SVG image
3. `jsPDF.addImage()` per page
4. Output `ArrayBuffer`

**Alternative:** `window.print()` on styled HTML wrapper — user-triggered, not automated export.

### 9.3 MuseScore Fallback Detection

```
function select_backend(config: PdfExportConfig) -> PdfBackend:
    if config.backend == Verovio:
        return Verovio
    if config.backend == MuseScore:
        if not find_musescore(config.musescorePath):
            throw MuseScoreNotFound
        return MuseScore
    // auto
    if find_musescore(config.musescorePath):
        return MuseScore
    return Verovio
```

### 9.4 Incremental Preview (Pagination)

For large scores, render visible pages only:

```
visible_range = [currentPage - 1, currentPage + 1]
for page in visible_range:
    cache[page] ||= toolkit.renderToSVG(page)
```

Cache invalidated on composition revision ID change.

---

## 10. Interfaces

### 10.1 Tauri Commands

```typescript
// Generate MusicXML and render PDF
export_pdf(composition_id: string, config: PdfExportConfig): Promise<string /* path */>

// Preview: return SVG for single page (Verovio only)
render_score_page(composition_id: string, page: number, scale: number): Promise<string /* svg */>

// Get page count without full render
get_score_page_count(composition_id: string): Promise<number>

// Check MuseScore availability
detect_musescore(): Promise<{ found: boolean; path?: string; version?: string }>
```

### 10.2 Vue Components

| Component | Responsibility |
|-----------|----------------|
| `ScorePreview.vue` | Verovio SVG display, page navigation |
| `PdfExportDialog.vue` | Backend selection, quality preset |
| `ExportProgress.vue` | Async job progress (shared with MIDI/MusicXML) |

### 10.3 Events

```typescript
// Tauri event: pdf-render-progress
{ job_id: string, page: number, total_pages: number }
```

---

## 11. Parameter Mappings

| User Parameter | PDF Effect |
|----------------|------------|
| `export.pdf.backend` | Verovio / MuseScore / auto |
| `export.pdf.page_size` | A4 / Letter |
| `export.engraving_quality` | Maps to backend + scale |
| `style.orchestration_preset` | Part count → page count estimate |
| `form.section_count` | Page break hints via MusicXML `<print>` |

### 11.1 MusicXML → Renderer Feature Support

| MusicXML Feature | Verovio | MuseScore |
|------------------|---------|-----------|
| Multi-part | ✓ | ✓ |
| Chord symbols | ✓ | ✓ |
| Cross-staff beaming | △ | ✓ |
| Pedal lines | △ | ✓ |
| Ottava | ✓ | ✓ |
| Rehearsal marks | ✓ | ✓ |
| `aurora:*` attributes | Ignored | Ignored |

---

## 12. Explainability Model

PDF is presentation-only. The UI provides **click-through from PDF preview to Inspector** only when using in-app Verovio preview backed by live AST — not from exported PDF file.

Export audit log:

```json
{
  "format": "pdf",
  "backend": "verovio",
  "page_count": 12,
  "musicxml_profile": "publish",
  "render_ms": 847
}
```

---

## 13. Future Expansion

| Phase | Feature |
|-------|---------|
| v0.2 | Web Worker for Verovio (non-blocking UI) |
| v0.2 | PNG/SVG per-page export |
| v0.3 | Layout template presets (lead sheet, full score, piano reduction) |
| v0.4 | Server-side headless Chromium for CI golden PDF tests |

---

## 14. Open Questions

1. Bundle Verovio WASM in Tauri app vs CDN lazy load?
2. Default MuseScore path detection per OS — registry vs PATH?
3. Copyright footer in PDF (`footer: encoded`) — legal requirement?
4. Single-page continuous scroll vs discrete pages in preview UI?

---

## 15. References

- Verovio documentation: https://book.verovio.org/
- verovio-toolkit-wasm npm package
- MuseScore 4 command-line interface handbook
- [MusicXML Export Specification](musicxml.md)
- [ADR-004](../../decisions/ADR-004-musicxml-primary-export.md)
- [Export Research Notes](../../research/export-research-notes.md)
- Elaine Gould, *Behind Bars* (engraving reference)

---

*End of PDF Rendering Specification v0.1*
