# AXIOM Native Backend

## Current role

The native backend is transition infrastructure inherited from the historical AXIOM implementation.

It currently lowers the existing native pipeline into a standalone executable by packaging an AXIOM artifact with the Rust runtime.

## Historical architecture

```text
AXIOM source
    ↓
Rust lexer/parser
    ↓
Rust semantic analysis
    ↓
AXIOM_ARTIFACT_V1
    ↓
Rust AOT launcher
    ↓
Rust runtime
    ↓
standalone executable
```

This architecture is useful for bootstrap and testing, but it is not the final AXIOM 0.1 architecture.

## Target architecture

```text
AXIOM source
    ↓
AXIOM compiler
    ↓
AXIOM IR
    ↓
native target backend
    ↓
AXIOM runtime / target machine
```

The target backend must be defined by AXIOM semantics rather than by Rust-specific implementation assumptions.

## Target model

Targets may include:

- desktop/server CPUs;
- SIMD;
- GPU;
- NPU;
- embedded systems;
- mobile;
- web;
- distributed systems;
- specialized hardware;
- future execution targets.

The language remains one language. A target changes compilation strategy, available capabilities, and constraints.

## ABI

The current executable boundary is process-oriented and Rust-hosted.

AXIOM 0.1 requires a language-owned ABI covering representation, layout, alignment, calls, structures, errors, resources, capabilities, and compatibility.

## Reproducibility

Transition builds are deterministic at the serialized artifact level where declared inputs are identical. Final executable byte identity may still depend on the host Rust compiler and platform linker.

This dependency is one of the boundaries targeted by the self-hosting program.
