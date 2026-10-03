# AXIOM SYSTEMS

AXIOM is a universal, general-purpose programming language and technology ecosystem designed around a small visible syntax and a deep semantic model.

> Natural in expression, formal in meaning.

This repository is the implementation workspace for AXIOM. The repository is currently in the transition from a historical prototype to the implementation of the definitive AXIOM language defined by the AXIOM 0.1 specification.

## Current status

The conceptual architecture of AXIOM 0.1 is closed. The next work is implementation: progressively replacing the historical Python and Rust language implementations with AXIOM-owned compiler, runtime, tooling, and eventually operating-system foundations.

The repository therefore contains two layers:

1. **Transition infrastructure** — the existing Python and Rust implementations, native artifact format, runtime, package workflow, and bootstrap experiments.
2. **AXIOM 0.1 implementation program** — the new canonical language frontend, semantic model, compiler architecture, self-hosting path, ABI, runtime, and toolchain described in the current specification.

The transition is intentional. Historical code is not the definition of the language; the specification is.

## Target architecture

The long-term compiler path is:

```text
AXIOM source
    ↓
AXIOM compiler
    ↓
semantic model / plan
    ↓
AXIOM IR
    ↓
native backend / runtime
    ↓
target machine
```

The self-hosting end state is:

```text
AXIOM source
    ↓
AXIOM compiler written in AXIOM
    ↓
AXIOM compiler
    ↓
AXIOM compiles AXIOM
```

Python and Rust are temporary implementation and bootstrap technologies. They may remain during migration, but neither is part of the language definition or the intended final compiler foundation.

## AXIOM 0.1 language direction

AXIOM 0.1 is built around:

- entities, relations, resources, capabilities, effects, restrictions, contracts, knowledge, and execution plans;
- indentation-based blocks;
- a deliberately small visible syntax;
- semantic dependency declarations rather than textual inclusion as the core module model;
- automatic resource-flow analysis;
- information-flow analysis;
- inferred effects and capability provenance;
- compiler-directed concurrency;
- explicit hard constraints and soft preferences;
- first-class units, time, determinism, verification, and failure semantics;
- native access from application level to systems and hardware programming;
- a stable AXIOM ABI and FFI model;
- deterministic builds and supply-chain metadata;
- progressive self-hosting and toolchain independence.

Canonical examples:

```axiom
mostrar("Hello, AXIOM")

usuario:
    nombre: texto
    edad: entero

alex: usuario
    nombre = "Alex"
    edad = 25

sumar(a, b):
    a + b

decidir edad >= 18:
    verdadero:
        mostrar("Adult")
    falso:
        mostrar("Minor")
```

The canonical syntax is being implemented incrementally. It must not be confused with the historical prototype syntax still present in compatibility tests and bootstrap code.

## Repository structure

- `docs/axiom-spec-0.1.md` — definitive conceptual specification for AXIOM 0.1.
- `docs/axiom-language.md` — canonical syntax and executable-language design status.
- `docs/architecture.md` — compiler, semantic, IR, runtime, and toolchain architecture.
- `docs/self-hosting.md` — migration from bootstrap infrastructure to AXIOM self-hosting.
- `docs/implementation-status.md` — factual implementation status and remaining boundaries.
- `docs/version-roadmap.md` — AXIOM 0.1 → 1.0 implementation sequence.
- `ROADMAP.md` — language/toolchain milestone history and current implementation roadmap.
- `bootstrap/` — historical and transitional self-hosting experiments.
- `src/axiom/` — historical Python implementation and transition frontend.
- `native/` — historical/current Rust-native transition implementation.
- `tests/` — Python compatibility and subsystem tests.
- `native/tests/` — native and end-to-end transition tests.
- `examples/` — examples retained for the currently implemented transition syntax.

The master roadmap remains in `Documento sin título.txt`. It is intentionally not edited by implementation work in this repository.

## Historical milestones

The original language/toolchain roadmap phases 1–18 have been completed as documented historical milestones. They established:

- a working prototype compiler pipeline;
- AXIOM IR and executable artifacts;
- a native Rust toolchain;
- a bootstrap seed written in AXIOM;
- a self-hosted bootstrap frontend and compiler driver;
- a native project/package workflow.

These milestones are foundations for the next stage, not proof that the definitive AXIOM 0.1 language is already self-hosted.

## Development

Until the AXIOM compiler can build itself, the transition toolchain is used to validate the new implementation.

Python:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
python -m pytest
```

Rust transition toolchain:

```bash
cargo fmt --manifest-path native/Cargo.toml -- --check
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path native/Cargo.toml
```

Do not treat these commands as the final AXIOM developer workflow. They are temporary transition infrastructure.

## Independence policy

The project is moving through explicit implementation boundaries:

```text
Historical Python/Rust implementation
        ↓
AXIOM 0.1 frontend in AXIOM
        ↓
AXIOM 0.1 semantic compiler in AXIOM
        ↓
AXIOM compiler compiles itself
        ↓
AXIOM-owned runtime and native backend
        ↓
AXIOM-owned toolchain
        ↓
AXIOM 1.0
```

A dependency is removed only after the replacement is implemented, tested, reproducible, and able to rebuild the required component.

## Branch policy

Active implementation work is performed on the dedicated branch `local/continue-project`. The `main` branch is not modified by this development workflow.

No remote push is performed unless explicitly requested.

## License

AXIOM SYSTEMS is distributed under the [MIT License](LICENSE).
