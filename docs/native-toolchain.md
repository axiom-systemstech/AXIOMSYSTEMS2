# AXIOM Native Toolchain

Phase 18 makes the native axiom binary the reference developer workflow.

The binary is self-contained at runtime: it parses AXIOM, performs semantic
validation, builds AXIOM_ARTIFACT_V1, executes programs, runs project tests,
creates packages and installs itself. Python is not required by these commands.

## Project workflow

    axiom new hello
    axiom check hello
    axiom build hello
    axiom run hello
    axiom test hello
    axiom package hello -o hello.axpkg

A generated project contains axiom.toml, src/main.ax, and tests/main.ax.

## Native artifact

axiom build writes build/main.axm. The artifact is versioned as
AXIOM_ARTIFACT_V1 and can be executed directly:

    axiom run hello/build/main.axm

axiom native-build creates a standalone executable. The executable contains the
AXIOM artifact and the native runtime and does not require the AXIOM CLI or
Python after creation.

## Installation

The compiled CLI can install itself into a prefix:

    axiom install /opt/axiom

This creates bin/axiom. Distribution packages can therefore ship the native
binary and runtime without the historical Python frontend.
## Package manager

The native package workflow uses a deterministic AXIOM_PACKAGE_V1 container.
Files are sorted by relative path and encoded as hexadecimal records, so the
same project content produces the same package bytes.

    axiom package hello -o hello.axpkg

Local registry packages can be vendored without Python:

    axiom add my-project networking registry

The native bootstrap records the selected version in axiom.lock. Remote
registries, signatures and cryptographic package verification remain separate
distribution concerns.

## Targets

The native backend accepts explicit Rust target triples:

    axiom native-build examples/hello.ax -o out/hello --target aarch64-unknown-linux-gnu

The requested target must be installed in the native build environment.
## Independence boundary

The Phase 18 boundary is:

    AXIOM source
        ↓
    native axiom toolchain
        ↓
    AXIOM_ARTIFACT_V1
        ↓
    native runtime / standalone executable

Python is no longer a runtime or command-line prerequisite. Rust remains the
implementation language of the current bootstrap host and native runtime; the
next architectural step would be replacing that host boundary with AXIOM-owned
native code rather than treating Rust as the language foundation.
