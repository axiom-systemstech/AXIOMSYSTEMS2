# Contributing to AXIOM

## Development branch

Active implementation work uses:

```text
local/continue-project
```

Do not develop directly on `main` for this implementation program.

Remote pushes require explicit authorization.

## Source of truth

Use the following order when making implementation decisions:

1. `docs/axiom-spec-0.1.md`
2. `docs/axiom-language.md`
3. `docs/architecture.md`
4. `docs/self-hosting.md`
5. `docs/version-roadmap.md`
6. `ROADMAP.md`
7. `Documento sin título.txt` for the wider AXIOM SYSTEMS program.

The master roadmap is read-only for this repository work.

## Implementation rule

Do not preserve historical behavior merely because the prototype already implements it.

When historical behavior conflicts with the canonical AXIOM 0.1 design, migrate the implementation toward the specification and add a conformance test for the intended behavior.

## Milestones

Prefer small, meaningful commits.

A milestone should normally contain:

- implementation;
- tests;
- documentation;
- reproducibility evidence where relevant.

Avoid mixing unrelated refactors with language-semantic changes.

## Validation

Before a milestone is considered complete, run the relevant checks:

```bash
python -m pytest
cargo fmt --manifest-path native/Cargo.toml -- --check
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path native/Cargo.toml
git diff --check
```

Use additional conformance tests for canonical AXIOM 0.1 syntax and semantics as they become available.

## Self-hosting discipline

The historical Python and Rust implementations are bootstrap infrastructure.

Do not create new permanent compiler architecture that depends on either implementation unless the dependency is explicitly documented as a temporary bootstrap boundary.

Every removed dependency must be replaced by an implementation that is tested and reproducible.

## Documentation

Documentation is part of the implementation.

When a semantic decision changes:

1. update the canonical specification;
2. update the language documentation;
3. update the relevant architecture or roadmap document;
4. add or update executable conformance tests.

## Code style

Keep compiler layers explicit and small.

Prefer deterministic behavior.

Diagnostics should explain the semantic problem and identify the source location.

Do not silently guess ambiguous programmer intent.

## Pull requests and integration

Before merging a branch into `main`, verify:

- the implementation matches the canonical specification;
- tests pass;
- generated artifacts are reproducible where required;
- documentation is current;
- the master roadmap was not modified unintentionally.

No push is implied by local completion.
