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
- Run the current editor buffer through the AXIOM compiler and runtime.
- Run all project AXIOM sources as an integrated test action.
- Inspect Git status from the workspace.
- Build an `.axpkg` package from the workspace.
- No external web framework or editor dependency.

The Studio server is intended for local development. Source editing currently operates on the in-memory browser buffer; the Run action executes that buffer without writing it to disk. Project actions operate on the selected workspace.

## Roadmap

Debugger, profiler, integrated terminal, richer package management, Git actions, testing controls, documentation navigation, and visual tooling will be added incrementally on top of this workspace and language-service foundation.
