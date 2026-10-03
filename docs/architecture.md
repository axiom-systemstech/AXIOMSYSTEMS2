# AXIOM Architecture

## Purpose

This document defines the implementation architecture for the definitive AXIOM language.

The architecture is organized around semantic understanding rather than around the historical Python/Rust parser split.

## Compiler model

The canonical pipeline is:

```text
Source
  ↓
Lexical representation
  ↓
Syntax / entity representation
  ↓
Semantic model
  ↓
Resource Flow
  ↓
Information Flow
  ↓
Effects and capabilities
  ↓
Contracts and restrictions
  ↓
Execution plan
  ↓
AXIOM IR
  ↓
Target backend
  ↓
Runtime / machine
```

The compiler may fuse or reorder internal passes for performance, but the semantic boundaries remain explicit.

## Core semantic model

The internal model contains:

- VALUE;
- DATA;
- RESOURCE;
- CAPABILITY;
- KNOWLEDGE.

Important relations include:

- produce;
- consume;
- transform;
- read;
- write;
- depend;
- call;
- communicate;
- control;
- create;
- release;
- contain;
- measure.

The compiler uses these relations to derive effects, resource lifetimes, parallelism, restrictions, and target decisions.

## Frontend

The frontend will be implemented progressively in AXIOM itself.

Responsibilities:

- lexical analysis;
- canonical syntax parsing;
- entity/form construction;
- source locations;
- syntax diagnostics;
- module discovery;
- semantic name resolution.

The current Python `new_parser.py` is a migration bridge only. It must not become the permanent compiler frontend.

## Semantic compiler

The semantic compiler resolves:

- types;
- relations;
- modules;
- packages;
- capabilities;
- effects;
- resource flow;
- information flow;
- restrictions;
- contracts;
- execution feasibility.

AXIOM 0.1 now has an explicit semantic-facts layer (`semantic_model.py`) between the AST and the historical IR. It records entity kinds, semantic relations, resources, capabilities, and effects, and propagates function effects through the call graph.
Resource Flow in this layer uses an explicit state lattice and conservative joins. Resource identities can cross resource-typed function parameters and return through those parameters without changing identity; structured resource fields are modeled as contained identities and retain their state across calls; nested indexed expressions remain views rather than aliases and are also represented as contained subresources.

Ambiguity is diagnosed rather than guessed.

## Intermediate representation

AXIOM IR is the stable internal boundary between semantic compilation and execution backends.

The IR should represent semantic intent sufficiently to support:

- native CPU execution;
- SIMD;
- GPU/NPU/QPU targets;
- embedded targets;
- distributed execution;
- real-time planning;
- verification metadata;
- explainability.

The historical `AXIOM_ARTIFACT_V1` format remains a transition artifact. It is not the final AXIOM ABI.

## Runtime

The runtime owns operations that cannot be reduced entirely to static compilation:

- resource acquisition/release;
- scheduling;
- device interaction;
- I/O;
- process and machine boundaries;
- recovery;
- dynamic runtime state.

The runtime must expose explicit semantic boundaries rather than leaking host-language assumptions into AXIOM.

## Native backend

The native backend maps AXIOM execution plans to target machines.

The target model is:

```text
AXIOM program
    ↓
semantic execution plan
    ↓
target-specific lowering
    ↓
machine code / runtime calls
```

Rust currently hosts this transition layer. The final architecture is not Rust-specific.

## ABI and FFI

AXIOM will define an ABI covering:

- value representation;
- layout;
- alignment;
- calling convention;
- structures;
- errors;
- resource handles;
- capabilities;
- versions;
- compatibility.

Interop targets include C, C++, Rust, Python, .NET, WebAssembly, native APIs, and hardware interfaces.

WebAssembly is an interoperability target, not the language foundation.

## Toolchain

The final toolchain will be implemented from AXIOM-owned components:

```text
axiom new
axiom check
axiom build
axiom run
axiom test
axiom package
axiom add
axiom install
axiom native-build
axiom explain
axiom format
axiom lint
axiom migrate
```

The historical native Rust CLI is a transition implementation of part of this surface.

## Build identity and reproducibility

A build is identified by:

```text
project + dependencies + compiler + configuration + target
```

Build outputs must be deterministic where the target and declared inputs permit determinism.

Supply-chain metadata includes origin, identity, version, integrity, dependencies, capabilities, effects, supported platforms, ABI information, and reproducibility information.

## Explainability boundary

Compiler explanations are generated from compiler state:

- semantic facts;
- inferred effects;
- resource graph;
- capability graph;
- execution plan;
- target constraints;
- optimization decisions;
- verification evidence.

No external service is required.

## Migration rule

When the current implementation conflicts with the canonical specification, implementation is migrated toward the specification.

The specification is not rewritten merely to preserve an accidental property of the prototype.
