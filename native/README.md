# AXIOM Native

This crate is the first native backend component of AXIOM SYSTEMS. It mirrors
the Python lexer and the first parser slice and is intentionally independent
from the Python bootstrap so both implementations can be compared before
migration.

```bash
cargo test --manifest-path native/Cargo.toml
```