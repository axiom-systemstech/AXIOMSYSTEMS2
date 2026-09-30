# AXIOM Studio

AXIOM Studio is the local development environment introduced in Phase 12 of the roadmap. The first Studio milestone is deliberately dependency-free and reuses the AXIOM compiler frontend rather than maintaining a second language implementation.

## Launch

From an AXIOM project:

```text
axiom studio
```

The default address is `http://127.0.0.1:8765`. A different project, host, or port can be selected with:

```text
axiom studio path/to/project --host 127.0.0.1 --port 9000
```

## Current capabilities

- Project workspace discovery from `axiom.toml` and `.ax` files.
- Browser-based source editor.
- Source diagnostics using the real AXIOM parser and semantic analyzer.
- Basic language completion for keywords, types, built-ins, functions, and structs.
- No external web framework or editor dependency.

The Studio server is intended for local development. It does not expose project write operations yet; editing currently operates on the in-memory browser buffer and diagnostics are performed against that buffer.

## Roadmap

Debugger, profiler, integrated terminal, package management, Git integration, testing controls, documentation navigation, and visual tooling will be added incrementally on top of this workspace and language-service foundation.
