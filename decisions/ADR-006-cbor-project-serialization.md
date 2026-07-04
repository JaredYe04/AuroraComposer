# ADR-006: CBOR for Project File Serialization

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)

## Context

Project persistence (`score.md`) must serialize the full Composition AST, parameter bundle, patch history, and optional IR cache. IPC between Tauri and Vue uses JSON; disk storage format was undecided (REV open question #1).

## Decision

Use **CBOR** (Concise Binary Object Representation) for `.aurora` project bundle files. Use **JSON** for Tauri IPC only.

## Consequences

- Smaller project files (~3× compression vs JSON per ast.md analysis)
- Faster load/save for large compositions
- `serde_cbor` crate in `aurora-ast`
- Human debugging via `aurora-cli dump --format json` (Phase 2 utility)

## Alternatives Considered

- **JSON everywhere:** Simple but large files and slower parse
- **MessagePack:** Similar to CBOR; less standard tooling
- **SQLite:** Overkill for v0.1 single-user desktop
