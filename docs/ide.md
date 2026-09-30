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

Studio now provides the complete Phase 12 development surface:

- source editor, diagnostics, and completion
- Run and workspace testing actions
- source-level execution tracing with function/instruction breakpoints
- function and instruction profiling
- Git status and diff inspection
- local package manifest, lockfile, vendor inspection, and package installation
- a restricted integrated terminal for project-safe development commands
- documentation index and document loading from the workspace
- AXIOM IR visualization
- local `.axpkg` packaging

The debugger and profiler operate on the AXIOM IR executed by the runtime. Debug traces expose function entry/exit and instruction events, while configured breakpoints report matching function/instruction locations. Profiling reports elapsed execution time, function calls, and instruction counts.

The integrated terminal intentionally exposes a fixed allowlist rather than arbitrary shell execution. Studio remains bound to `127.0.0.1` by default.

Studio remains dependency-free and uses the real AXIOM compiler, semantic analyzer, IR, runtime, package manager, and Git tooling rather than parallel implementations.

The Studio server is intended for local development. Source editing currently operates on the in-memory browser buffer; the Run action executes that buffer without writing it to disk. Project actions operate on the selected workspace.

## Next phase

Phase 12 establishes Studio as the local development environment for the current AXIOM toolchain. The next roadmap layer is AXIOM Core, which can consume these compiler, runtime, package, testing, and tooling interfaces without introducing a second development stack.
