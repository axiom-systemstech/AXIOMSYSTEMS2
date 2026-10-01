# AXIOM Native Backend

## Architecture

The native backend sits after the existing native parser and semantic analysis:

```text
AXIOM source
    ↓
native lexer/parser
    ↓
native semantic analysis
    ↓
AXIOM native artifact
    ↓
AOT launcher generation
    ↓
rustc + axiom_native runtime
    ↓
standalone executable
```

The generated executable embeds the serialized AXIOM artifact and links the native
runtime as an `rlib`. The executable therefore does not require Python or the AXIOM
CLI at runtime.

## CLI

```bash
cargo run --manifest-path native/Cargo.toml -- native-build examples/hello.ax
cargo run --manifest-path native/Cargo.toml -- native-build examples/hello.ax out/hello
cargo run --manifest-path native/Cargo.toml -- native-build examples/hello.ax out/hello --target aarch64-unknown-linux-gnu
```

`host` is the default target. A non-host target is passed directly to Cargo and
rustc, so the corresponding Rust target and linker/toolchain must be installed.

## ABI boundary

The current native boundary is deliberately process-oriented: an AXIOM executable
has a native `main` entry point and communicates through the host process ABI and
standard I/O. The runtime itself remains an internal Rust library until a stable
AXIOM FFI ABI is defined in a later phase.

## Reproducibility

Native generation avoids source timestamps and incremental build state. The generated
launcher uses a fixed crate name, one codegen unit, fixed optimization settings,
and path remapping. The serialized AXIOM artifact is deterministic; final executable
byte identity also depends on the installed rustc and platform linker toolchain.

Cross-compilation is target-driven rather than host-driven: the requested target
triple determines the runtime library build and final executable target.
