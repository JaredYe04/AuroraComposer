# ADR-007: Style-Switch for Jazz vs Classical Harmony

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)

## Context

Jazz harmony rules (`JAZZ-*`) and classical functional harmony rules (`HARM-*`) can conflict at style boundaries (REV-002). Example: parallel fifths tolerated in jazz voicings but hard-forbidden in strict counterpoint mode.

## Decision

Implement a **style rule bundle switch** controlled by `style.genre` and `style.era`:

| Condition | Active Rule Bundles |
|-----------|---------------------|
| `style.genre ∈ {classical, baroque, romantic}` | `HARM-*`, `VL-*`, `CP-*` (strictness from params) |
| `style.genre ∈ {jazz, blues, fusion}` | `JAZZ-*`, `HARM-*` (subset), `VL-*` (relaxed) |
| `style.genre ∈ {pop, rock, electronic}` | `HARM-*`, `RHY-*`; CP optional |

**Rule `JAZZ-001` (HARD):** When `style.genre` is jazz-family, classical `HARM-PROG-*` diatonic templates are **disabled**; jazz progression graph is **enabled**.

**Rule `STYLE-001` (HARD):** Only one primary harmony vocabulary active per generation job. No mixing without explicit `style.hybrid_mode = true` (Phase 3).

Harmony Engine mode selection (DP vs beam) reads `harmony.complexity` independently of style bundle.

## Consequences

- Rule Engine loads bundle set at Style Resolver (Stage 1)
- No contradictory rules evaluated in same pass
- Inspector shows active bundle in composition metadata

## Alternatives Considered

- **Weighted merge of all rules:** Unpredictable scoring; rejected
- **Separate engines per style:** Duplication; rejected for v0.1
