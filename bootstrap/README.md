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

## Self-hosting milestone

`lexer.ax` and `parser.ax` are now compiler frontend subsystems executed from
AXIOM itself. The lexer reads `lexer_fixture.ax` and emits a deterministic token
stream; the parser consumes that stream and walks a broad AST subset covering
structs, typed parameters and returns, arrays, literals, calls, field/index
access, binary/unary expressions, assignments, conditionals, loops and control
flow.

The native VM exposes only the host boundaries required by this stage:
`read_file`, `write_file`, `split`, `split_lines`, `char_at`, and `char_code`.

## Verify

```bash
cargo run --manifest-path native/Cargo.toml -- check bootstrap/seed.ax
cargo run --manifest-path native/Cargo.toml -- run bootstrap/seed.ax
cargo run --manifest-path native/Cargo.toml -- run bootstrap/lexer.ax
cargo run --manifest-path native/Cargo.toml -- check bootstrap/parser.ax
cargo test --manifest-path native/Cargo.toml
```

The self-hosted lexer is covered by a native integration test and produces the same
stream on every run for the fixture source.

## Scope

This is a self-hosting milestone, not full self-hosting. The lexer and parser
frontend have moved into AXIOM, but the current parser emits a deterministic textual
AST representation rather than the native typed AST. Structured AST construction,
semantic analysis, IR lowering, and compiler-driver integration still remain to be
migrated. Rust remains the bootstrap host until AXIOM can build its own complete
compiler source tree reproducibly.
