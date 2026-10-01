# AXIOM Native

This crate is the first native backend component of AXIOM SYSTEMS. It mirrors
the Python lexer and the first parser slice and is intentionally independent
from the Python bootstrap so both implementations can be compared before
migration. The current native pipeline covers lexing, parsing, and the first
semantic checks.

The native runtime executes the current AXIOM language slice, while the AOT backend packages that runtime with a compiled AXIOM artifact into a standalone executable. The native pipeline now supports explicit Rust target triples and deterministic output configuration.

```rust
axiom_native::runtime::run("fn main() { print(\"Hello AXIOM\") }")?;
```

```bash
cargo test --manifest-path native/Cargo.toml
```

Build and use the native CLI:

```bash
cargo build --manifest-path native/Cargo.toml
native/target/debug/axiom check examples/hello.ax
native/target/debug/axiom run examples/hello.ax
native/target/debug/axiom native-build examples/hello.ax out/hello
native/target/debug/axiom native-build examples/hello.ax out/hello-aarch64 --target aarch64-unknown-linux-gnu
```

Array literals and chained indexing are supported by the native pipeline. Array
types use postfix brackets, for example `fn first(values: Int[]) -> Int[]`.