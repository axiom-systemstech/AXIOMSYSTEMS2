# AXIOM Implementation Status

## Status date

This document describes the repository state at the beginning of the definitive AXIOM 0.1 implementation program.

## What is complete

The historical language/toolchain program has established:

- lexical analysis and parsing;
- semantic analysis;
- functions and control flow;
- arrays and structures;
- AXIOM IR;
- VM/runtime execution;
- CLI;
- standard-library foundation;
- package workflow;
- project workflow;
- AXIOM Studio foundation;
- AXIOM Core;
- AXIOM Engine foundation;
- native AOT packaging;
- bootstrap seed;
- self-hosted bootstrap lexer/parser/AST/semantic/compiler stages;
- native project/package workflow;
- deterministic package/artifact paths.

The repository also contains the consolidated AXIOM 0.1 semantic specification.

## Current 0.1 implementation state

The canonical AXIOM 0.1 English frontend is now executable and covered by conformance tests. It supports the stable source kernel used by the repository: expressions, assignments, indentation blocks, decisions, repetition over collections, structures, typed entities, function definitions, calls, and final-expression functions.

The historical brace-based parser remains available as transition infrastructure.

The following are still design commitments or transition infrastructure rather than complete AXIOM 1.0 capabilities:

- complete AXIOM 0.1 semantic compiler;

- complete Resource Flow implementation;
- complete Information Flow enforcement;
- complete effects/capabilities system;
- complete contracts and verification engine;

Implemented slice: canonical `needs`, `can`, `prefer`, `restrict`, `prove`, and `mode` directives survive parsing into the AST and are exposed by the semantic model. Effect-derived capabilities become requirements, explicit `can` declarations constrain them, and unauthorized inferred capabilities produce diagnostics. Resource-like values emit explicit creation relations, lifecycle calls (`release`, `close`, `free`, `drop`) emit release relations, and the semantic model tracks resource state, consumers, release provenance, and use-after-release diagnostics. This is an implementation slice, not the final Resource Flow or verification engine.
Implemented slice: Resource Flow now has an explicit four-state lattice (`ACTIVE`, `RELEASED`, `MAYBE_RELEASED`, `SHARED`) with conservative branch/loop joins. Resource identities transfer through resource parameters across function calls, release effects propagate back to callers, returned parameter resources preserve identity, and nested indexed views are distinguished from aliases. These behaviors are covered by semantic-model tests. Resource Flow remains incomplete for closures, slices/subranges, copy/move/reuse semantics, resource containers with typed resource elements, concurrency, distribution, devices, and complete normative verification. Structured resource fields are modeled as contained resource identities, including nested fields and interprocedural release propagation.
- compiler-directed execution planning;
- final AXIOM IR;
- final AXIOM ABI;
- AXIOM-owned runtime;
- AXIOM-owned native backend;
- AXIOM-owned package/toolchain implementation;
- complete FFI;
- complete distribution model;
- production metaprogramming/reflection;
- complete standard library;
- AXIOM compiler self-build using the canonical language;
- removal of Python as a compiler dependency;
- removal of Rust as the compiler/runtime language foundation.

## Historical implementations

### Python

The Python implementation remains useful as transition infrastructure and as a fast testing environment.

It is not the intended final compiler implementation.

### Rust

The Rust implementation is the current native transition host. It provides the existing parser, semantic pipeline, IR, runtime, VM, package workflow, and standalone executable generation.

It is not the definition of AXIOM and is not the intended final language foundation.

### AXIOM bootstrap

The `bootstrap/` directory contains the first AXIOM-written compiler experiments.

These experiments are evidence that AXIOM can participate in its own bootstrap, but they target the historical artifact/runtime boundary and therefore are not yet the definitive AXIOM 0.1 compiler.

## Current canonical source of truth

Use these documents in this order:

1. `docs/axiom-spec-0.1.md` — semantic and architectural definition.
2. `docs/axiom-language.md` — canonical syntax direction.
3. `docs/architecture.md` — implementation architecture.
4. `docs/self-hosting.md` — independence path.
5. `docs/version-roadmap.md` — release and implementation gates.
6. `ROADMAP.md` — language/toolchain milestone history.
7. `Documento sin título.txt` — wider AXIOM SYSTEMS master roadmap.

The master roadmap is intentionally left untouched by this repository update.

## Acceptance rule

A feature is not considered implemented merely because it is described.

A feature becomes complete when its:

- syntax;
- semantics;
- compiler implementation;
- runtime/backend behavior;
- diagnostics;
- tests;
- reproducibility characteristics;
- documentation

agree with the canonical specification.
