# AXIOM Language Roadmap

This roadmap tracks the implementation of the AXIOM language and its toolchain. The master AXIOM SYSTEMS roadmap is maintained separately in `Documento sin título.txt` and is intentionally not modified here.

## Current position

**Historical language/toolchain phases 1–18: complete.**

Those phases established a functional prototype, native transition toolchain, bootstrap, self-hosted bootstrap frontend, and native project workflow.

**Current program: AXIOM 0.1 implementation and progressive self-hosting.**

The conceptual design is closed. Implementation now proceeds against `docs/axiom-spec-0.1.md`.

## Completed foundation

- [x] Phase 1 — language fundamentals.
- [x] Phase 2 — semantic system.
- [x] Phase 3 — control flow.
- [x] Phase 4 — data types and collections.
- [x] Phase 5 — structs and data model.
- [x] Phase 6 — AXIOM IR.
- [x] Phase 7 — runtime and VM.
- [x] Phase 8 — CLI and basic toolchain.
- [x] Phase 9 — standard library foundation.
- [x] Phase 10 — package manager foundation.
- [x] Phase 11 — project workflow.
- [x] Phase 12 — AXIOM Studio foundation.
- [x] Phase 13 — AXIOM Core.
- [x] Phase 14 — AXIOM Engine foundation.
- [x] Phase 15 — native backend.
- [x] Phase 16 — bootstrap seed.
- [x] Phase 17 — self-hosted bootstrap frontend and compiler driver.
- [x] Phase 18 — native ecosystem workflow.

## Definitive AXIOM implementation program

The following stages supersede the old assumption that the historical prototype itself is the final language.

### AXIOM 0.1 — canonical language foundation

Goal: make the new semantic architecture executable.

- [ ] Canonical lexer in AXIOM.
- [ ] Canonical parser in AXIOM.
- [ ] Canonical AST/entity model.
- [ ] Canonical names, scopes, modules, and semantic resolution.
- [ ] Core type system and unit/dimensionality model.
- [ ] Canonical Resource Flow model.
- [ ] Effects and capability analysis.
- [ ] Information Flow foundations.
- [ ] Canonical diagnostics.
- [ ] AXIOM 0.1 conformance tests.

### AXIOM 0.2 — compiler semantics

Goal: compile meaningful AXIOM 0.1 programs.

- [ ] Semantic planning pipeline.
- [ ] Contracts and verification model.
- [ ] Control flow and block-as-value semantics.
- [ ] Collections, entities, functions, modules, and packages.
- [ ] Deterministic lowering to AXIOM IR.
- [ ] Explainable compiler decisions.
- [ ] Canonical standard-library interface.

### AXIOM 0.3 — self-hosted compiler

Goal: AXIOM becomes the primary implementation language of its compiler.

- [ ] Compiler frontend implemented in AXIOM.
- [ ] Semantic compiler implemented in AXIOM.
- [ ] IR generator implemented in AXIOM.
- [ ] Compiler driver implemented in AXIOM.
- [ ] AXIOM compiler builds AXIOM compiler source.
- [ ] Reproducible self-build comparison.

### AXIOM 0.4 — native execution foundation

Goal: reduce and then remove the historical Rust compiler/runtime boundary.

- [ ] AXIOM-owned runtime interfaces.
- [ ] AXIOM ABI implementation.
- [ ] Native backend contracts.
- [ ] Resource and memory execution model.
- [ ] Platform target model.
- [ ] Native bootstrap path that does not require Python.

### AXIOM 0.5 — autonomous toolchain

Goal: the AXIOM compiler, package manager, formatter, diagnostics, tests, and build workflow are implemented from AXIOM-owned components.

- [ ] Compiler CLI.
- [ ] Package resolution.
- [ ] Reproducible builds.
- [ ] Formatter.
- [ ] Linter.
- [ ] Test runner.
- [ ] Explainability tooling.
- [ ] Migration tooling.
- [ ] Documentation generation.

### AXIOM 0.6–0.9 — production language maturation

These releases are implementation milestones, not promises of specific calendar dates. They will stabilize:

- [ ] concurrency and execution planning;
- [ ] real-time and determinism;
- [ ] fault tolerance and recovery;
- [ ] distribution;
- [ ] metaprogramming and reflection;
- [ ] FFI and ABI;
- [ ] security and information-flow enforcement;
- [ ] advanced optimization;
- [ ] cross-platform targets;
- [ ] standard-library breadth;
- [ ] ecosystem and package verification.

Each feature is accepted only when its semantics, implementation, tests, diagnostics, and documentation agree.

### AXIOM 1.0 — independence milestone

AXIOM 1.0 is the point at which the language and its core toolchain can be built and maintained by AXIOM itself without requiring Python or Rust as the language implementation foundation.

The 1.0 gate requires evidence, not intent:

- [ ] AXIOM compiler builds from AXIOM source.
- [ ] Compiler self-build is reproducible.
- [ ] Core runtime is AXIOM-owned or uses only explicitly defined external ABI boundaries.
- [ ] Core toolchain no longer depends on the historical Python implementation.
- [ ] Core toolchain no longer depends on Rust as its language implementation.
- [ ] Canonical AXIOM 1.0 syntax and semantics have conformance tests.
- [ ] Native targets are validated.
- [ ] Package and build identities are reproducible.
- [ ] Migration and compatibility rules are documented.
- [ ] The compiler can explain its relevant resource, effect, capability, and optimization decisions.

## Independence boundary

The migration is:

```text
Python/Rust prototype
        ↓
AXIOM-owned frontend
        ↓
AXIOM-owned compiler
        ↓
AXIOM compiles AXIOM
        ↓
AXIOM-owned runtime/backend
        ↓
AXIOM-owned toolchain
        ↓
AXIOM 1.0
```

Rust and Python may remain as temporary bootstrap hosts until each boundary has a tested replacement. Removing them prematurely would make the bootstrap less reliable rather than more independent.

## Master roadmap relationship

`Documento sin título.txt` remains the authoritative master roadmap for the wider AXIOM SYSTEMS program. This file does not declare any master-roadmap phase complete merely because a language/toolchain milestone is complete.
