# AXIOM Bootstrap Seed

The bootstrap seed is the first compiler component implemented in AXIOM itself.
It deliberately starts from a small, deterministic subset instead of duplicating
the Rust frontend all at once.

## Pipeline

```text
Token[]
  ↓
parse_print()
  ↓
Program
  ↓
emit_program()
  ↓
AXIOM_ARTIFACT_V1
```

The seed models tokens and an AST-level program using AXIOM structs, performs a
real lowering step for a `print` program, and emits the native artifact format.
The emitted artifact can be consumed by the existing native VM.

## Verify

```bash
cargo run --manifest-path native/Cargo.toml -- check bootstrap/seed.ax
cargo run --manifest-path native/Cargo.toml -- run bootstrap/seed.ax
```

The expected emitted artifact is deterministic for the same source and toolchain.

## Scope

This is the bootstrap starting point, not full self-hosting. File I/O, a complete
AXIOM lexer/parser, semantic analysis, IR lowering, and compiler-driver integration
still remain to be migrated from Rust. Those migrations are the work of the next
self-hosting phase.
