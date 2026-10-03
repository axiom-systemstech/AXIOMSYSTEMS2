# AXIOM Self-Hosting

## Objective

Self-hosting means that AXIOM can compile and maintain the implementation of its own compiler without requiring Python or Rust as the language implementation foundation.

This is the main engineering objective of the current development stage.

## Current boundary

The repository currently contains:

```text
Historical Python frontend/runtime
        +
Historical Rust native compiler/runtime
        +
AXIOM bootstrap programs
```

The previous self-hosting phase proved that AXIOM can execute bootstrap compiler components and generate deterministic intermediate output.

That is an important bootstrap milestone, but it is not yet definitive AXIOM 0.1 self-hosting.

## Target boundary

The target is:

```text
AXIOM source
    ↓
AXIOM compiler written in AXIOM
    ↓
AXIOM IR
    ↓
AXIOM-owned runtime/backend
    ↓
target machine
```

Eventually the compiler source itself follows the same path:

```text
AXIOM compiler source
    ↓
existing AXIOM compiler
    ↓
new AXIOM compiler
    ↓
new AXIOM compiler compiles itself
```

## Migration stages

### Stage A — canonical frontend

Implement the AXIOM 0.1 lexer and parser in AXIOM.

The Python frontend may validate behavior during this stage, but Python is not the final implementation.

### Stage B — canonical semantic model

Implement in AXIOM:

- entities;
- forms/types;
- scopes;
- modules;
- relations;
- Resource Flow;
- effects;
- capabilities;
- Information Flow;
- restrictions;
- diagnostics.

### Stage C — canonical compiler

Implement the semantic compiler and AXIOM IR generator in AXIOM.

The compiler must be able to compile real AXIOM 0.1 programs, not only bootstrap fixtures.

### Stage D — self-build

The AXIOM compiler compiles its own source.

The result must be compared against a clean rebuild and must be deterministic under identical inputs.

### Stage E — Python removal

Python ceases to be required for:

- compiling AXIOM;
- checking AXIOM;
- running the compiler;
- building projects;
- executing the core toolchain.

The historical Python implementation may remain as archival test material until removal is safe.

### Stage F — Rust boundary reduction

Rust stops being the language implementation foundation.

Rust can remain temporarily as a low-level bootstrap host while AXIOM-owned runtime and backend components are introduced.

### Stage G — autonomous toolchain

The compiler, package manager, formatter, linter, test runner, explainability tools, migration tools, and documentation generation are AXIOM-owned.

## Bootstrap policy

Bootstrap dependencies are allowed while they are actively being replaced.

A dependency is removed only when:

1. the replacement exists;
2. the replacement passes the relevant tests;
3. the replacement can reproduce the required artifacts;
4. the old dependency is no longer required by the target workflow.

This avoids a false sense of independence.

## Reproducibility gate

Every self-hosting milestone should record:

- compiler source revision;
- compiler used for the build;
- dependency set;
- target;
- generated artifact identity;
- deterministic comparison result;
- test result.

A self-hosting claim without reproducible evidence is not considered complete.

## Final independence

The final milestone is not "AXIOM can run one AXIOM program."

The final milestone is:

> AXIOM can build the AXIOM compiler, runtime, and core toolchain from AXIOM-owned source through a reproducible bootstrap path.

That is the independence boundary toward AXIOM 1.0.
