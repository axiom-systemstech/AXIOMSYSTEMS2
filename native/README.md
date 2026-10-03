# AXIOM Native Transition Layer

This crate contains the current Rust-native transition implementation of AXIOM.

It provides:

- the historical native lexer/parser;
- semantic analysis;
- AXIOM artifact generation;
- the transition VM/runtime;
- native project workflow;
- package workflow;
- standalone executable generation.

## Important boundary

Rust is currently the native implementation host.

Rust is **not** the definition of AXIOM and is not the intended final language implementation foundation.

The project is progressively replacing this boundary with AXIOM-owned compiler and runtime components.

## Current commands

```bash
cargo test --manifest-path native/Cargo.toml

cargo build --manifest-path native/Cargo.toml

native/target/debug/axiom check examples/hello.ax
native/target/debug/axiom run examples/hello.ax
native/target/debug/axiom native-build examples/hello.ax out/hello
```

## Transition status

The native toolchain is useful for bootstrapping the definitive compiler and validating low-level execution.

It should be treated as migration infrastructure until the AXIOM-owned compiler, ABI, runtime, and backend replace its language-implementation responsibilities.
