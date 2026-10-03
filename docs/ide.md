# AXIOM Studio

AXIOM Studio is the local development environment established during the historical Phase 12 milestone.

## Current capabilities

The transition Studio provides:

- source editing;
- diagnostics;
- completion;
- run and testing actions;
- execution tracing;
- breakpoints;
- profiling;
- Git inspection;
- package inspection;
- integrated terminal;
- documentation browsing;
- IR visualization;
- local package creation.

## Architectural rule

Studio must consume the real AXIOM compiler, semantic model, IR, runtime, package manager, and diagnostics.

It must not become a second language implementation.

## Future direction

As the canonical AXIOM compiler becomes self-hosted, Studio will move to the canonical compiler interfaces.

The long-term Studio should expose semantic information directly:

- resources;
- capabilities;
- effects;
- information flows;
- execution plans;
- verification evidence;
- target decisions;
- diagnostics;
- migration diffs.

The current Studio is transition infrastructure and is not yet the final AXIOM 0.1 development environment.
