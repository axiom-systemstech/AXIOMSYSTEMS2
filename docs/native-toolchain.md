# AXIOM Native Toolchain

The native toolchain is the current transition implementation of the AXIOM project workflow.

## Current workflow

```text
axiom new
axiom check
axiom build
axiom run
axiom test
axiom package
axiom add
axiom native-build
axiom install
```

The current native binary can execute this workflow without the Python CLI.

## Project artifacts

A project contains an `axiom.toml` manifest, source files, and tests.

The current implementation produces:

- `AXIOM_ARTIFACT_V1` executable artifacts;
- `AXIOM_PACKAGE_V1` package archives;
- standalone native executables.

These formats are transition formats.

## Independence boundary

The current boundary is:

```text
AXIOM source
    ↓
Rust-native transition toolchain
    ↓
AXIOM artifact
    ↓
Rust runtime / executable
```

Python is not required for the supported native workflow.

Rust remains the implementation host.

The next architectural boundary is:

```text
AXIOM source
    ↓
AXIOM compiler
    ↓
AXIOM IR
    ↓
AXIOM-owned runtime/backend
```

## Targets

The transition backend accepts explicit native target triples.

This is useful infrastructure, but target selection must eventually become part of the AXIOM semantic target model rather than a Rust-specific command-line detail.
