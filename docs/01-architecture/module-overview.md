# Module Overview

**Document:** Aurora Composer — Module Overview  
**Version:** 0.1  
**Status:** Draft

---

## 1. Module Map

All modules listed below are **architectural boundaries**. Each will have a dedicated specification before implementation.

```text
┌─────────────────────────────────────────────────────────────┐
│                        FRONTEND MODULES                      │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ ParamPanel   │ TimelineView │ PianoRoll    │ Inspector      │
│ ExportDialog │ Player       │ PluginManager│ ProjectBrowser │
└──────────────┴──────────────┴──────────────┴────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                      TAURI SHELL MODULES                   │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ CommandRouter│ JobManager   │ ProjectStore │ PluginHost     │
│ FileService  │ ConfigStore  │ EventBus     │ Capabilities   │
└──────────────┴──────────────┴──────────────┴────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                    COMPOSITION ENGINE MODULES                │
├─────────────────────────────────────────────────────────────┤
│ PipelineOrchestrator                                         │
├──────────────┬──────────────┬──────────────┬────────────────┤
│StyleResolver │EmotionResolver│StructEngine  │ThemePlanner    │
├──────────────┼──────────────┼──────────────┼────────────────┤
│HarmonyEngine │MelodyEngine  │RhythmEngine  │CounterpointEng │
├──────────────┼──────────────┼──────────────┼────────────────┤
│BassEngine    │DrumEngine    │DecorEngine   │RepairEngine    │
├──────────────┴──────────────┴──────────────┴────────────────┤
│ ValidationEngine                                             │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                   INFRASTRUCTURE MODULES                     │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ MusicAST     │ MusicIR      │ RuleEngine   │ SearchEngine   │
│ TheoryCore   │ Provenance   │ ParamMapper  │ ConstraintSolv │
└──────────────┴──────────────┴──────────────┴────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                      EXPORT MODULES                          │
├──────────────┬──────────────┬──────────────┬────────────────┤
│ MusicXMLExp  │ MIDIExporter │ ABCExporter  │ PDFRenderer    │
│ IRProjector  │ AudioPreview │              │                │
└──────────────┴──────────────┴──────────────┴────────────────┘
```

---

## 2. Module Catalog

### 2.1 Music Model

| Module | Purpose | Input | Output | Spec |
|--------|---------|-------|--------|------|
| **MusicAST** | Canonical tree representation | Patches | AST nodes | [ast.md](../02-music-model/ast.md) |
| **MusicIR** | Flat export representation | AST | IR events | [ir.md](../02-music-model/ir.md) |
| **TheoryCore** | Pitch, interval, scale utilities | Pitch classes | Intervals, chords | [harmony.md](../03-theory/harmony.md) |
| **Provenance** | Event metadata tracking | Rule results | Provenance records | [ast.md](../02-music-model/ast.md) |
| **ParamMapper** | User params → weights | Parameters | Weight table | ACAS §6 |

### 2.2 Pipeline Stages

| Module | Purpose | AST Read | AST Write | Spec |
|--------|---------|----------|-----------|------|
| **StyleResolver** | Genre → params + plugins | — | Metadata | [pipeline.md](pipeline.md) |
| **EmotionResolver** | Emotion → weights | — | Metadata | [emotion-engine.md](../04-algorithms/emotion-engine.md) |
| **StructureEngine** | Form, sections, phrases | Global | Section tree | [structure-engine.md](../04-algorithms/structure-engine.md) |
| **ThemePlanner** | Theme allocation | Sections | Theme refs | [motif-engine.md](../04-algorithms/motif-engine.md) |
| **HarmonyEngine** | Chord progressions | Sections | Chord events | [harmony-engine.md](../04-algorithms/harmony-engine.md) |
| **RhythmEngine** | Metric patterns | Measures | Rhythm attrs | [rhythm.md](../03-theory/rhythm.md) |
| **MelodyEngine** | Melodic generation | Chords + rhythm | Note events | [melody-engine.md](../04-algorithms/melody-engine.md) |
| **CounterpointEngine** | Additional voices | Melody + chords | Voice events | [counterpoint.md](../03-theory/counterpoint.md) |
| **BassEngine** | Bass line | Harmony | Bass voice | [harmony-engine.md](../04-algorithms/harmony-engine.md) |
| **DrumEngine** | Percussion | Structure + rhythm | Drum voice | [drum-engine.md](../04-algorithms/drum-engine.md) |
| **DecorEngine** | Ornaments | Melody voices | Note attrs | [melody-engine.md](../04-algorithms/melody-engine.md) |
| **RepairEngine** | Fix violations | Full AST | Patches | [repair-engine.md](../04-algorithms/repair-engine.md) |
| **ValidationEngine** | Hard constraint check | Full AST | Report | [constraint.md](../05-rule-engine/constraint.md) |

### 2.3 Infrastructure

| Module | Purpose | Spec |
|--------|---------|------|
| **RuleEngine** | Evaluate rules against AST | [rule-dsl.md](../05-rule-engine/rule-dsl.md) |
| **ConstraintSolver** | Hard/soft constraint checking | [constraint.md](../05-rule-engine/constraint.md) |
| **SearchEngine** | Beam/A*/DP search | [scoring.md](../05-rule-engine/scoring.md) |
| **PipelineOrchestrator** | Stage sequencing, progress | [pipeline.md](pipeline.md) |

### 2.4 Export

| Module | Purpose | Spec |
|--------|---------|------|
| **IRProjector** | AST → IR transformation | [ir.md](../02-music-model/ir.md) |
| **MusicXMLExporter** | IR → MusicXML | [musicxml.md](../06-export/musicxml.md) |
| **MIDIExporter** | IR → SMF | [midi.md](../06-export/midi.md) |
| **ABCExporter** | IR → ABC notation | [abc.md](../06-export/abc.md) |
| **PDFRenderer** | MusicXML → PDF | [pdf.md](../06-export/pdf.md) |
| **AudioPreview** | IR → audio buffer | research/playback.md |

### 2.5 Application Shell

| Module | Purpose | Spec |
|--------|---------|------|
| **CommandRouter** | Tauri IPC dispatch | [tauri.md](../08-ui/tauri.md) |
| **JobManager** | Async generation jobs | [backend.md](backend.md) |
| **ProjectStore** | Project persistence | [backend.md](backend.md) |
| **PluginHost** | Plugin lifecycle | [api.md](../07-plugin/api.md) |

### 2.6 Frontend

| Module | Purpose | Spec |
|--------|---------|------|
| **ParamPanel** | Parameter controls | [vue.md](../08-ui/vue.md) |
| **TimelineView** | Section/measure timeline | [timeline-ui.md](../08-ui/timeline-ui.md) |
| **PianoRoll** | Note-level editing | [piano-roll.md](../08-ui/piano-roll.md) |
| **Inspector** | Event provenance display | [inspector.md](../08-ui/inspector.md) |
| **Player** | Audio playback | [vue.md](../08-ui/vue.md) |

---

## 3. Module Dependencies

```text
TheoryCore ← all algorithm engines
MusicAST ← all pipeline stages, UI, export
RuleEngine ← all generative stages
SearchEngine ← MelodyEngine, HarmonyEngine, CounterpointEngine
ParamMapper ← all pipeline stages
PluginHost ← StyleResolver, all plugin-type engines
IRProjector ← all exporters, AudioPreview
JobManager ← PipelineOrchestrator
CommandRouter ← JobManager, ProjectStore, PluginHost
```

---

## 4. Module Communication Rules

1. **Pipeline stages** communicate only through AST mutations
2. **Rule Engine** is stateless; receives AST snapshot, returns scores
3. **Search Engine** manages transient state; commits winner to AST
4. **Frontend** never accesses engine internals; only Tauri commands
5. **Plugins** implement documented traits; loaded by PluginHost
6. **Exporters** read IR only; never trigger generation

---

## 5. Future Modules (Post-v0.1)

| Module | Phase | Notes |
|--------|-------|-------|
| AIPluginHost | Phase 3 | ML model inference sandbox |
| CollaborationSync | Phase 3 | Multi-user AST editing |
| MicrotonalEngine | Phase 3 | Extended tuning systems |
| VocalEngine | Phase 3 | Melismatic line generation |
| DAWBridge | Phase 3 | VST/AU plugin export |

---

## References

- [Architecture](architecture.md)
- [Pipeline](pipeline.md)
- [ACAS v0.1](../00-overview/acas-v0.1.md)
