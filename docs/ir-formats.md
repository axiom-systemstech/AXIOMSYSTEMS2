# AXIOM Intermediate Representations

## Current transition formats

The repository currently contains two historical serialized formats:

- `.air` — textual bootstrap IR;
- `.axm` — textual native VM artifact.

Both are versioned and validated by their current consumers.

## AXIOM_ARTIFACT_V1

The current native artifact is headed by:

```text
AXIOM_ARTIFACT_V1
```

It stores compiled functions and encoded VM instructions for the transition runtime.

It is deterministic for identical compiler inputs under the current implementation.

## AXIOM 0.1 IR direction

The definitive AXIOM IR must represent semantic execution plans rather than merely mirror the historical VM instruction set.

It must be capable of carrying information relevant to:

- values and entities;
- resource flow;
- effects;
- capabilities;
- restrictions;
- contracts;
- execution dependencies;
- parallelism;
- target selection;
- memory placement;
- verification evidence;
- distributed execution where required.

The exact stable representation will be frozen only after executable AXIOM 0.1 semantics are implemented.

## Compatibility

Language versions and IR format versions are separate.

A language change requires an IR version change only when existing consumers cannot correctly interpret the resulting representation.

The historical `.air` and `.axm` formats must not constrain the final AXIOM IR architecture.
