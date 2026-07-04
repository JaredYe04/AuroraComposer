# Theory System Research Notes

**Version:** 0.1  
**Date:** 2026-07-04  
**Agent:** Theory System Research Agent  
**Status:** Raw research notes supporting `docs/03-theory/` specifications

---

## 1. Research Scope

Parallel research pass covering seven theory domains for Aurora Composer Phase 1:

| Domain | Output spec | Rule prefix | Rules cataloged |
|--------|-------------|-------------|-----------------|
| Harmony | [harmony.md](../docs/03-theory/harmony.md) | HARM | 78 |
| Counterpoint | [counterpoint.md](../docs/03-theory/counterpoint.md) | CP | 62 |
| Voice leading | [voice-leading.md](../docs/03-theory/voice-leading.md) | VL | 47 |
| Rhythm | [rhythm.md](../docs/03-theory/rhythm.md) | RHY | 54 |
| Form | [form.md](../docs/03-theory/form.md) | FORM | 58 |
| Jazz | [jazz.md](../docs/03-theory/jazz.md) | JAZZ | 56 |
| Orchestration | [orchestration.md](../docs/03-theory/orchestration.md) | ORCH | 40 |
| **Total** | | | **395** |

---

## 2. Source Bibliography

### Primary textbooks (consulted for rule extraction)

| Author | Work | Used for |
|--------|------|----------|
| **Piston** | *Harmony* (5th ed.) | Functional harmony, orchestration |
| **Piston** | *Counterpoint* | Species overview |
| **Piston** | *Orchestration* | Ranges, doubling |
| **Kostka & Payne** | *Tonal Harmony* (8th ed.) | Progressions, form, rhythm examples |
| **Hindemith** | *Craft of Musical Composition* Vol. I | Interval hierarchy, chord classification |
| **Hindemith** | *Traditional Harmony* | Two-voice writing |
| **Aldwell, Schachter, Cadwallader** | *Harmony and Voice Leading* | VL rules, tendency tones |
| **Fux** | *Gradus ad Parnassum* (Mann/Kerman) | Species counterpoint CP-S1–S5 |
| **Levine** | *The Jazz Theory Book* | ii–V–I, extensions, voicings |
| **Nettles & Graf** | *Chord Scale Theory* | Jazz melody compatibility |
| **Coker** | *Hearin' the Changes* | Turnarounds, substitutions |
| **Caplin** | *Analyzing Classical Form* | Sonata, sectional form |
| **Schoenberg** | *Fundamentals of Musical Composition* | Theme development |
| **Adler** | *The Study of Orchestration* | Instrument ranges |
| **Rimsky-Korsakov** | *Principles of Orchestration* | Texture, balance |

### Datasets and tools

| Resource | Application |
|----------|-------------|
| **Bach 371 Chorales** | Validation corpus for HARM, CP, VL rules |
| **Groove MIDI Dataset** (Google/Magenta, 13.6h, 1150 files) | RHY-GRO pattern library extraction |
| **Lakh MIDI** | Progression frequency statistics (future) |
| **Weimar Jazz Database** | Jazz progression templates |
| **Music21** | Analysis benchmark (parallel fifths, RN analysis) |
| **Real Book / iReal Pro** | Jazz standard progressions |

### Internal Aurora documents

- [deep-research-report.md](../deep-research-report.md) — algorithm choice, parameter mapping, pipeline
- [acas-v0.1.md](../docs/00-overview/acas-v0.1.md) — rule engine overview
- [terminology.md](../docs/00-overview/terminology.md) — normative glossary
- [research-methodology.md](../docs/00-overview/research-methodology.md) — required spec sections

---

## 3. Hard vs Soft Classification Methodology

Rules classified using decision tree:

```text
Is violation musically impossible / unplayable / breaks export?
  YES → HARD (search prune)
  NO → Is it style-dependent with pedagogical exceptions?
    YES → SOFT with preset override to HARD (e.g. counterpoint.strictness)
    NO → SOFT (weighted penalty)
```

### Cross-domain HARD rules (high priority for Rule Engine)

| Rule | Domain | Rationale |
|------|--------|-----------|
| CP-PAR-001/002 | Counterpoint | Parallel P5/P8 at strictness ≥ 0.7 |
| HARM-066 | Harmony | No harmony change mid-tie |
| RHY-MTR-001 | Rhythm | Grid alignment |
| ORCH-RNG-001 | Orchestration | Absolute instrument range |
| JAZZ-VOICE-001 | Jazz | Guide tones in comping |
| VL-X-004 | Voice leading | Voice overlap SATB |

### Parameter → HARD promotion

| Parameter | Effect |
|-----------|--------|
| `counterpoint.strictness` → 1.0 | CP-PAR-*, VL-MOT-010/011 become HARD |
| `style=chorale_strict` | HARM-VOICE-009/011 HARD |
| `style=jazz` | JAZZ-001 activates jazz pack; classical HARM-PROG demoted |

---

## 4. Domain Research Findings

### 4.1 Harmony

- **Functional grammar** outperforms pure Markov for cadence guarantee (aligns with deep research report).
- **Two-phase** design (skeleton + voicing) reduces search space ~10× vs simultaneous chord+voice search.
- **Kostka & Payne** progression tables mapped to HARM-PROG-001–015 edges.
- **Hindemith** used for dissonance accent rules only; functional analysis retained for classical preset.
- Bach chorale analysis: ~0% parallel P5/P8 in corpus → validates CP-PAR strict mode target.

### 4.2 Counterpoint

- **Fux species** provide pedagogical HARD constraints; florid (5th species) is default generation target.
- **Motion preference** (contrary > oblique > similar) encoded as SOFT weights CP-020–028.
- 2-voice checker is O(n); 4-voice is O(6n) per beat — feasible for desktop beam search.
- **Hidden parallels** (CP-PAR-008/009) controversial; kept SOFT with outer-voice emphasis per Jeppesen.

### 4.3 Voice Leading

- Separated from counterpoint per Aldwell/Schachter: VL = successive; CP = simultaneous.
- **Common tone retention** (VL-CT-001) highest-impact SOFT rule for chorale quality.
- **Range rules** duplicated minimally with ORCH-RNG; VL-RNG for abstract voices, ORCH-RNG for instruments.

### 4.4 Rhythm

- **Groove MIDI** statistics: 4/4 dominates (~85% of dataset); rock/pop backbeat on 2+4 confirmed.
- **Template tiling** + 25% variation (RHY-GRO-006) balances repetition vs interest.
- **Swing ratio** 0.58–0.67 for jazz; parameterized not HARD.
- Latin clave (RHY-GRO-014) HARD only when `style=latin` — avoids false positives in pop.

### 4.5 Form

- **Caplin** formal functions mapped to FORM-SON-* for classical; pop uses FORM-SEC-*.
- **Theme similarity metric** (contour + interval + rhythm) threshold 0.6 triggers bridge (FORM-ABA-005).
- **Development operators** ordered by `development_level`: repetition → sequence → fragmentation.

### 4.6 Jazz

- **ii–V–I** is backbone; 13 rules in JAZZ-IIV including turnarounds.
- **Tritone substitution** (JAZZ-SUB-001) must preserve resolution (HARD JAZZ-SUB-002).
- **Rootless voicings** when bass voice present — Levine convention.
- **Conflict with harmony.md:** jazz preset supersedes HARM-PROG via JAZZ-001 HARD switch.

### 4.7 Orchestration

- **40 rules** — smallest catalog; mostly range HARD + texture SOFT.
- **SATB_choral** default preset until full orchestral CSP in Phase 2.
- **Double bass 8vb** (ORCH-RNG-009) HARD — critical for MIDI export pitch accuracy.

---

## 5. Parameter Mapping Summary (Cross-Domain)

Consolidated from specs + deep research report:

| User parameter | Primary domains | Internal effect |
|----------------|-----------------|-----------------|
| `complexity` | HARM, JAZZ, CP | Extensions, species level, secondary dominants |
| `counterpoint.strictness` | CP, VL | HARD promotion threshold |
| `cadence_strength` | HARM, FORM | HARM-CAD weights, FORM-PHR-002 |
| `harmonic_rhythm` | HARM, JAZZ | Change rate |
| `rhythm.density` | RHY, ORCH | Pattern density, accompaniment subordination |
| `rhythm.syncopation` | RHY | RHY-SYNC index |
| `rhythm.swing` | RHY, JAZZ | Groove ratio |
| `theme_count` | FORM | Theme allocation |
| `texture` | ORCH, CP | Homophony vs polyphony activation |
| `orchestration_preset` | ORCH | Range tables, VoiceGroup |
| `style` | All | Rule pack selection |

---

## 6. Engineering Recommendations

1. **Single Rule Registry** loading all prefixes; filter by active style pack.
2. **Unified parallel detector** shared by CP-PAR and VL (avoid duplicate logic).
3. **Provenance chain** must include all fired rule IDs per event (ACAS principle 2).
4. **Phased implementation:** HARM triads + VL core → CP 2-voice → RHY templates → FORM pop → JAZZ → ORCH SATB.
5. **Validation suite:** Bach chorales + 10 Real Book leads + Groove MIDI pattern replay.

---

## 7. Conflicts and Open Issues

| Issue | Documents | Proposed resolution |
|-------|-----------|---------------------|
| Jazz vs classical progression | harmony.md, jazz.md | JAZZ-001 style switch |
| VL vs ORCH range duplication | voice-leading.md, orchestration.md | VL abstract; ORCH instrument-specific |
| P4 consonance above bass | counterpoint.md | 2-voice HARD; 3+ voice SOFT |
| Hindemith vs functional dissonance | harmony.md | Weight by preset |
| AST schema pending | All | Block Rule DSL formalization until ast.md Wave 1 |

---

## 8. Dependency Updates for TASKS.md

New downstream tasks identified:

- [ ] Rule Engine: import 395 rules into rule-dsl.md schema
- [ ] Harmony Engine: implement HARM two-phase algorithm
- [ ] Bach chorale regression test spec in testing.md
- [ ] Groove MIDI offline pattern extraction script (research, not production)
- [ ] Resolve AST Chord node schema before voicing implementation

---

## 9. Document Statistics

| File | Approx. lines | Sections | Rules |
|------|---------------|----------|-------|
| harmony.md | ~550 | 15 + 5 appendices | 78 |
| counterpoint.md | ~480 | 15 + 5 appendices | 62 |
| voice-leading.md | ~320 | 15 + 1 appendix | 47 |
| rhythm.md | ~380 | 15 + 4 appendices | 54 |
| form.md | ~400 | 15 + 3 appendices | 58 |
| jazz.md | ~420 | 15 + 8 appendices | 56 |
| orchestration.md | ~350 | 15 + 4 appendices | 40 |

Combined theory specification: ~2,900 lines (~45–55 printed pages at standard formatting). Depth target met via rule catalogs, algorithms, data structures, and cross-references.

---

## 10. Next Steps

1. Architecture Agent consistency review against pending Music AST spec
2. Merge rule IDs into unified `rules/` index (future)
3. ADR if jazz/classical switch mechanism needs formal decision
4. Wave 4 algorithm specs reference these theory rules by ID

---

## References (Quick Links)

- [Harmony Spec](../docs/03-theory/harmony.md)
- [Counterpoint Spec](../docs/03-theory/counterpoint.md)
- [Voice Leading Spec](../docs/03-theory/voice-leading.md)
- [Rhythm Spec](../docs/03-theory/rhythm.md)
- [Form Spec](../docs/03-theory/form.md)
- [Jazz Spec](../docs/03-theory/jazz.md)
- [Orchestration Spec](../docs/03-theory/orchestration.md)
- [Deep Research Report](../deep-research-report.md)
