# AXIOM Native

This crate is the first native backend component of AXIOM SYSTEMS. It mirrors
the Python lexer and the first parser slice and is intentionally independent
from the Python bootstrap so both implementations can be compared before
migration. The current native pipeline covers lexing, parsing, and the first
semantic checks.

The native runtime can execute the initial literal-printing slice:

```rust
axiom_native::runtime::run("fn main() { print(\"Hello AXIOM\") }")?;
```

```bash
cargo test --manifest-path native/Cargo.toml
```