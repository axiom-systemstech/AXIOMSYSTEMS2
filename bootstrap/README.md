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
`read_file`, `write_file`, `split`, `split_lines`, `char_at`, `char_code`, and
`int_to_string`.

`ast.ax` is the structured-AST normalization layer. It consumes the parser's
deterministic output and emits `AXIOM_AST_V1` records in the form
`NODE|depth|kind|value`. Scope depth is derived from explicit `ScopeEnter` and
`ScopeExit` nodes rather than from formatting whitespace.

`semantic.ax` is the self-hosted semantic bootstrap layer. It consumes
`AXIOM_AST_V1`, tracks function scopes, parameters and local declarations,
resolves variables across nested scopes, checks function and struct references,
requires `main`, and performs deterministic literal expression type checks.

`compiler.ax` is the self-hosted compiler driver. It consumes the normalized AST
and emits `AXIOM_IR_V1` records from AXIOM itself. The native host is deliberately
kept below this boundary: it executes AXIOM bytecode and owns the final
`AXIOM_ARTIFACT_V1` serialization/runtime ABI. A native integration test builds
the AST of `compiler.ax`, runs `compiler.ax` twice, and requires identical IR.

## Verify

```bash
cargo run --manifest-path native/Cargo.toml -- check bootstrap/seed.ax
cargo run --manifest-path native/Cargo.toml -- run bootstrap/seed.ax
cargo run --manifest-path native/Cargo.toml -- run bootstrap/lexer.ax
cargo run --manifest-path native/Cargo.toml -- check bootstrap/parser.ax
cargo test --manifest-path native/Cargo.toml
```

The self-hosted lexer is covered by a native integration test and produces the same
stream on every run for the fixture source. The self-hosted compiler test additionally
requires deterministic `AXIOM_IR_V1` output when compiling `bootstrap/compiler.ax`.

## Scope

Phase 17 is complete under the bootstrap contract: AXIOM owns the frontend stages,
bootstrap semantic analysis, compiler driver, and AST→`AXIOM_IR_V1` lowering. Rust
remains only as the execution host for AXIOM bytecode and the final artifact/runtime
ABI. Removing that remaining host boundary is explicitly deferred to Phase 18.
