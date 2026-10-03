# AXIOM CLI

The CLI documented here describes the transition toolchain and the intended future command surface.

## Current transition commands

```text
axiom doctor
axiom new <project>
axiom check <source.ax|project>
axiom build <source.ax|project> [-o <output>]
axiom run <source.ax|project|artifact.axm>
axiom test <project>
axiom package <project> [-o <output>]
axiom add <project> <package> [registry]
axiom native-build <source.ax|project> [-o <output>]
axiom install <prefix>
```

These commands are currently implemented primarily by the native Rust transition toolchain.

## Intended canonical surface

The final AXIOM toolchain will additionally provide:

```text
axiom explain
axiom format
axiom lint
axiom migrate
```

Their semantics are defined by the AXIOM 0.1 architecture and will be implemented progressively.

## Transition boundary

The native CLI can currently operate without the historical Python CLI for its supported workflow.

This is a significant milestone, but it does not make the whole AXIOM implementation independent of Rust. The Rust binary is still the current implementation host.

## Principle

CLI commands are interfaces to semantic compiler operations. They should not become separate implementations of language behavior.
