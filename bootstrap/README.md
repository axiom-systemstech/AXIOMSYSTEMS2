# AXIOM Bootstrap

The `bootstrap/` directory contains the first AXIOM-written compiler experiments.

## Historical pipeline

The bootstrap sequence established:

```text
source
  ↓
AXIOM-written lexer
  ↓
AXIOM-written parser
  ↓
AXIOM-written AST normalization
  ↓
AXIOM-written semantic bootstrap
  ↓
AXIOM-written compiler driver
  ↓
AXIOM_IR_V1 / AXIOM_ARTIFACT_V1
```

This proved that AXIOM can participate in its own bootstrap.

## What this does prove

- AXIOM can express compiler-oriented data structures.
- AXIOM can process source text.
- AXIOM can build structured representations.
- AXIOM can perform deterministic semantic checks.
- AXIOM can generate deterministic intermediate output.

## What this does not prove

It does not yet prove definitive AXIOM 0.1 self-hosting.

The bootstrap programs target the historical transition syntax and artifact/runtime boundary. The new AXIOM 0.1 compiler must be rebuilt around the canonical semantic model described in `docs/axiom-spec-0.1.md`.

## Verification

```bash
cargo test --manifest-path native/Cargo.toml
```

The historical integration tests verify deterministic bootstrap behavior.

## Next role

The bootstrap directory will become the bridge from the historical compiler to the canonical AXIOM compiler.

As AXIOM 0.1 implementation advances, components should move from bootstrap experiments into the canonical compiler architecture. Once a component is replaced and verified, the historical version can be retired.
