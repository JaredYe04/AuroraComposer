# ADR-002: Vue 3 Frontend (over React)

**Status:** Accepted  
**Date:** 2026-07-04

## Context

The deep research report suggested React + Tauri. The ACAS project direction specifies Vue 3 + Tauri for the frontend shell.

## Decision

Use **Vue 3 + TypeScript** for the Aurora Composer frontend, integrated with Tauri 2.x for desktop shell and IPC.

## Consequences

- UI specifications written for Vue composition API and Pinia state management
- Component library TBD (Naive UI or equivalent)
- Deep research report UI sections treated as technology-agnostic where possible

## Alternatives Considered

- **React:** Mentioned in research report; not selected per project direction
- **Svelte:** Lighter but smaller ecosystem for music visualization components
