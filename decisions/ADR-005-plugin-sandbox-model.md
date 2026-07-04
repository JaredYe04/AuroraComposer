# ADR-005: Plugin Sandbox Model

**Status:** Accepted  
**Date:** 2026-07-04  
**Accepted:** 2026-07-04 (Design Freeze v0.1)

## Context

Aurora Composer's plugin system must extend the composition engine with style-specific knowledge (harmony, rhythm, themes) and optional AI/export capabilities while preserving:

1. **AST centrality** — plugins read snapshots, return patches only
2. **Explainability** — every plugin mutation carries provenance
3. **Security** — third-party plugins must not access arbitrary filesystem, network, or host memory
4. **Performance** — native Rust plugins needed for search-heavy harmony/melody stages

The deep research report listed "Rust dynamic libs / WASM modules" as TBD. Plugin & UI research surveyed VST, WASM, Tauri capabilities, and subprocess isolation.

## Decision

Adopt a **tiered sandbox model** with three execution tiers:

| Tier | Name | Use Case | Isolation |
|------|------|----------|-----------|
| **T0** | Trusted Native | Bundled Aurora and partner-signed plugins | Capability-restricted in-process (`dlopen`) |
| **T1** | WASM Sandbox | Third-party algorithm plugins | `wasmtime` + host function API |
| **T2** | Subprocess | AI inference, untrusted experimental | Separate process, CBOR IPC |

### Tier Assignment Rules

- **Style, Harmony, Rhythm, Theme plugins** — T0 (signed) or T1 (community) based on manifest `trust_level`
- **AI plugins** — T2 mandatory (network + model loading)
- **Export plugins** — T0 or T1; file write via host-mediated `export_write` only

### Capability Manifest

Each plugin declares permissions in `aurora-plugin.json`:

```json
{
  "permissions": {
    "ast_read": ["Measure", "HarmonySlot", "Event"],
    "ast_write": ["Event", "HarmonySlot"],
    "network": false,
    "filesystem": [],
    "max_memory_mb": 256,
    "max_cpu_ms_per_call": 5000
  }
}
```

Host enforces permissions before and after each `apply()` call.

### Provenance Requirement

All plugin patches MUST include `ProvenanceAgent::Plugin { plugin_id }` on every created or modified Event. Host rejects patches missing provenance (invariant I-PROV-1).

## Consequences

### Positive

- Performance-critical bundled plugins run at native speed (T0)
- Community plugins cannot crash or exfiltrate data easily (T1/T2)
- Aligns with Tauri 2.x capability philosophy
- AI plugins isolated from composition engine memory

### Negative

- Three code paths increase Plugin Host complexity
- WASM AST serialization adds latency (~1–5 ms per call for typical compositions)
- T0 plugins can still crash host — mitigated by crash recovery and optional T1 migration
- Plugin developers must target WASM SDK for marketplace distribution

### Implementation Notes

- v0.1 prototype: T0 only for bundled plugins; T1 WASM in v0.2; T2 in Phase 3
- `PluginHost` trait unified across tiers; execution backend selected by manifest
- See [api.md](../docs/07-plugin/api.md) for trait definitions

## Alternatives Considered

### Single In-Process Native (Rejected)

All plugins as `.so`/`.dll` without sandbox. Simplest implementation but unacceptable security risk for third-party distribution and violates architecture security section.

### WASM Only (Rejected)

Maximum safety but 10–50× overhead unacceptable for beam search in harmony engine with thousands of candidate evaluations per stage.

### Electron-Style Node Sandbox (Rejected)

Not Rust-native; conflicts with Tauri stack decision; poor fit for CPU-intensive search.

### OS-Level Containers (Deferred)

Docker/WSL per plugin — too heavy for desktop UX. Revisit for cloud-side plugin execution in Phase 4.

## References

- [Plugin API Specification](../docs/07-plugin/api.md)
- [System Architecture §8 Security](../docs/01-architecture/architecture.md)
- [Plugin & UI Research Notes](../research/plugin-ui-research-notes.md)
- Tauri 2 Capabilities: https://v2.tauri.app/security/capabilities/
