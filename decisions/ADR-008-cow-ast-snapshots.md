# ADR-008: Copy-on-Write AST Snapshots During Search

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)

## Context

Beam search requires forking candidate states without O(n) deep clone per branch. ast.md §5.3 evaluated mutability models.

## Decision

Use **`Arc`-based copy-on-write snapshots** for search branches:

- `Composition` subtrees shared via `Arc`
- Branch fork: shallow clone + `Arc::make_mut` on modified path only
- Winning branch commits via atomic `Patch` to authoritative AST
- Search transient states never persist to project file

Optional Phase 3 optimization: persistent data structures (`im` crate).

## Consequences

- `aurora-ast` exposes `AstSnapshot::fork()` and `Patch::apply()`
- Provenance records `parent_state_id` linking search tree
- Memory bounded by beam_width × modified path depth

## Alternatives Considered

- **Full deep clone:** O(n) per branch; rejected
- **In-place mutation + rollback:** Thread-unsafe; rejected
