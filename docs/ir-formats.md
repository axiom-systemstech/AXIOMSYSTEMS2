# AXIOM Intermediate Formats

AXIOM uses two serialized intermediate representations during the current bootstrap and native pipeline:

- .air is the textual bootstrap IR.
- .axm is the textual native VM artifact.

Both formats carry an explicit format version. A producer must emit the version it implements, and a consumer must reject unsupported versions rather than silently interpreting them as a compatible format.

## .air

The current header is:

    AXIOM-IR 0.1

The remainder is a line-oriented representation of the lowered program.

A program with a main function starts with:

    FUNCTION main()

and ends with:

    END FUNCTION

Additional functions are emitted as separate FUNCTION blocks. Indentation represents nested control-flow blocks.

The current textual instructions include:

- LET
- SET
- PRINT
- CALL
- RETURN
- IF / ELSE / END
- WHILE / END
- FOR / UPDATE / END FOR
- BREAK
- CONTINUE

Expressions are rendered in AXIOM source notation.

The .air format is currently intended as a human-readable bootstrap representation. Its version must change when its grammar or instruction representation becomes incompatible with existing consumers.

## .axm

The current artifact header is:

    AXIOM_ARTIFACT_V1

Each compiled function is represented by:

    FUNCTION:<escaped-name>
    PARAMS:<count>
    PARAM:<escaped-name>
    INSTR:<encoded-instructions>

PARAM: occurs exactly count times.

Instructions are separated by semicolons. Nested control-flow instruction bodies are enclosed in brackets. String values and identifiers use the artifact escape rules implemented by the native VM serializer.

The native instruction set currently includes:

- scalar pushes
- variable loads and stores
- array construction and indexing
- indexed assignment
- struct construction
- struct field reads and writes
- arithmetic, comparison, logical and unary operations
- function calls
- output
- returns
- conditional and loop control flow
- break
- continue

Unknown headers, malformed instruction encodings and invalid counts are rejected during deserialization.

## Compatibility

Format versions are independent from the AXIOM language version.

A language change does not require a format version change when the serialized representation remains compatible. A format version must change when an existing consumer can no longer decode or correctly interpret the representation.

The native artifact currently has no backward-compatibility promise across major artifact versions. The versioned header exists so incompatible formats can be rejected explicitly.

The bootstrap .air representation is not a native executable format and is not consumed by the Rust VM.
