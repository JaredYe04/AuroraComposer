# ADR-001: Specification-First Development

**Status:** Accepted  
**Date:** 2026-07-04

## Context

Aurora Composer is a large-scale music composition engine with many interdependent modules (AST, rule engine, algorithm pipeline, export, UI, plugins). Ad-hoc implementation risks architectural drift, inconsistent terminology, and unmaintainable code.

## Decision

Adopt **Specification First** development: no production code until architecture documents reach Design Freeze.

Workflow: Research → Specification → Review → Freeze → Implementation.

The `docs/` directory is the single source of truth.

## Consequences

- Phase 1 duration extends (~18 weeks) but reduces rework
- All AI/human agents must read specs before coding
- Code conflicting with specs is considered incorrect
- Documentation target: 500–1000 pages before prototype

## Alternatives Considered

- **MVP-first:** Faster initial demo but incompatible with explainability and plugin goals
- **Code-as-spec:** Rejected; insufficient for multi-agent collaboration
