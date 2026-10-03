# AXIOM 0.1 — Consolidated Semantic Specification

**Status:** conceptual design closed; implementation is now underway.

This document is the semantic reference for the definitive AXIOM 0.1 implementation. It describes the language that the repository is moving toward, not every behavior of the historical prototype.

## 1. Identity

AXIOM is a universal, general-purpose, compiled programming language designed to express human intent naturally while preserving precise formal meaning.

> Natural in expression, formal in meaning.

AXIOM is one language across application, systems, embedded, scientific, robotics, industrial, distributed, hardware, and future operating-system domains.

The compiler must not depend on an AI model, network connection, external server, account, or remote service to compile, verify, optimize, or explain a program.

## 2. Semantic pipeline

The core semantic model is:

```text
INTENTION
    ↓
ENTITIES
    ↓
RELATIONS
    ↓
RESOURCES
    ↓
CAPABILITIES
    ↓
EFFECTS
    ↓
RESTRICTIONS
    ↓
CONTRACTS
    ↓
PLAN
    ↓
EXECUTION
```

Internal entity categories include:

- VALUE
- DATA
- RESOURCE
- CAPABILITY
- KNOWLEDGE

Core relations include:

- PRODUCE
- CONSUME
- TRANSFORM
- READ
- WRITE
- DEPEND
- CALL
- COMMUNICATE
- CONTROL
- CREATE
- RELEASE
- CONTAIN
- MEASURE

The compiler may use richer internal relations as implementation requires.

## 3. Canonical syntax

AXIOM 0.1 has one canonical human-facing syntax: English. The semantic core is language-neutral so future language frontends can provide Spanish and other human languages without creating a second semantic language.

The visible kernel is intentionally small:

```text
name:
name = expression
name(...)
if condition:
while condition:
repeat item in collection:
when condition:
```

Blocks use four-space indentation. Braces are not normal source delimiters.

Canonical examples:

```axiom
show("Hello, AXIOM")

name = "Alex"
age = 25
active = true

user:
    name: String
    age: Int

alex: user
    name = "Alex"
    age = 25

sum(a, b):
    a + b
```

Decision:

```axiom
if age >= 18:
    show("Adult")
else:
    show("Minor")
```

Repetition:

```axiom
repeat user in users:
    show(user.name)
```

Conditional event form:

```axiom
when temperature > 90:
    stop(motor)
```

The canonical frontend is executable and covered by conformance tests. Historical syntax such as `fn`, `print`, and brace-delimited blocks remains accepted only as transition syntax; it is not the canonical AXIOM 0.1 surface. Future Spanish support belongs to a language frontend, not to the core grammar.

## 4. Blocks and values

Blocks are semantic values.

A block normally produces the value of its final expression. Explicit return syntax is not a foundational language construct.

This allows the same block model to describe definitions, decisions, event responses, computations, resource plans, and other semantic constructs.

## 5. Types and forms

A type is formal knowledge about an entity.

Conceptually:

```text
TYPE =
    meaning
  + structure
  + identity
  + capabilities
  + relations
  + valid states
  + restrictions
  + contracts
  + representation
  + verifiable knowledge
```

The language distinguishes:

```text
TYPE
  ↓
FORM
  ↓
VALUE
  ↓
ENTITY
```

Core types include:

- boolean;
- integer;
- real;
- decimal;
- text;
- character;
- bytes;
- unit;
- time;
- duration.

Scientific types include:

- rational;
- complex;
- vector;
- matrix;
- tensor;
- interval;
- probability;
- algebraic numbers.

Units and dimensionality are first-class.

```axiom
mass = 5 kg
speed = 9.81 m/s²
temperature = 25 °C
period = 10 ms
```

Exact and approximate mathematics must remain distinguishable.

There is no universal null model. Absence and failure are semantic possibilities represented by the relevant type or contract.

## 6. Generalization

AXIOM does not make angle-bracket generics, inheritance, traits, or interface hierarchies the foundation of generalization.

Generalization is expressed through:

- semantic relations;
- conformance;
- required operations;
- restrictions;
- contracts;
- inference;
- specialization.

An abstraction should express the conditions it needs rather than enumerate every type it knows.

## 7. Modules and packages

The project hierarchy is:

```text
PROJECT → PACKAGE → MODULE → SEMANTIC SPACE → ENTITY
```

`usar` expresses semantic dependency.

`ofrecer` defines a public semantic interface.

Name resolution is deterministic. Resolution must not search arbitrary project state merely to guess programmer intent.

Resolution should consider, where relevant:

1. local definitions;
2. parameters and local entities;
3. current semantic space;
4. explicitly used modules;
5. official library entities;
6. declared package dependencies.

Ambiguity is an error, not an invitation to guess.

## 8. Resource Flow

Resource Flow replaces ownership/borrowing/lifetime syntax as the primary language model for resource management.

The compiler asks:

- what must exist;
- who needs it;
- for how long;
- what operations are allowed;
- what dependencies exist;
- when the last consumer finishes;
- what representation satisfies the constraints.

Conceptually:

```text
RESOURCE
   ↓
CREATION
   ↓
USES
   ↓
DEPENDENCIES
   ↓
LAST CONSUMER
   ↓
RELEASE / REUSE
```

The model applies to:

- memory;
- files;
- sockets;
- GPU/NPU/QPU buffers;
- devices;
- processes;
- threads;
- handles;
- connections;
- information;
- energy;
- time.

The compiler may choose copying, aliasing, views, moves, sharing, local storage, device storage, reuse, or other representations when semantics permit it.

Explicit restrictions have priority.

## 9. Information Flow

Resource Flow and Information Flow are separate analyses.

Information may carry classifications or policies.

A capability to write to a network does not automatically authorize sending every category of information through that network.

The compiler must be able to identify prohibited flows and explain their provenance.

## 10. Effects, capabilities, and resources

These are distinct:

```text
RESOURCE   = what an operation acts on
CAPABILITY = what it is authorized to do
EFFECT     = what it actually does
```

Effects are inferred and propagated transitively.

Examples:

```text
filesystem.read
filesystem.write
network.read
network.write
camera.read
gpu
clock.read
secure_randomness
hardware.raw_memory
```

`necesita` declares requirements.

`puede` declares allowed capabilities.

`prefiere` declares a soft preference.

`restringir` declares a hard constraint.

Capability provenance must be explainable: who introduced it, why it is required, where it is used, and which effects result.

## 11. Concurrency and execution planning

Concurrency is a property of the execution plan, not a separate asynchronous language.

The compiler may:

- discover independent operations;
- parallelize them;
- choose CPU/SIMD/GPU/NPU/QPU execution;
- distribute work across processes or machines;
- insert required synchronization;
- detect conflicts and races;
- preserve sequential execution when semantics require it.

Foundational syntax does not require `async`, `await`, `thread`, `mutex`, `lock`, or `spawn`.

A hard sequential constraint can be expressed semantically:

```axiom
restringir:
    ejecución = secuencial
```

## 12. Time and determinism

Time is an observable resource/effect.

The model distinguishes:

- instant;
- duration;
- clock;
- period;
- deadline;
- latency;
- jitter;
- priority.

Hard guarantees use restrictions. Soft objectives use preferences.

```axiom
modo:
    tiempo_real

restringir:
    periodo = 10 ms
    deadline = 5 ms
    determinismo = obligatorio
```

The compiler must distinguish:

- guaranteed/proven;
- feasible;
- not demonstrable;
- impossible under known constraints.

It must never claim a guarantee that it cannot establish.

## 13. Fault tolerance and recovery

Failure is a normal semantic possibility.

The model includes:

- retry;
- timeout;
- fallback;
- restart;
- isolate;
- degrade;
- replace;
- propagate;
- recover;
- cancel.

Retries require appropriate idempotency knowledge. The compiler must not blindly repeat operations with non-repeatable effects.

Resource cleanup and recovery are part of Resource Flow.

Distributed and hardware failure domains must be expressible.

## 14. Distribution

Distribution uses the same semantic model as local execution.

Additional semantic resources include:

- nodes;
- network links;
- serialization;
- latency;
- partitions;
- consistency;
- atomicity;
- recovery.

Contracts can describe:

- consistency;
- idempotency;
- atomicity;
- timeouts;
- retry safety;
- recovery behavior.

RPC, queues, sockets, and shared memory are implementation mechanisms, not separate language models.

## 15. Contracts and verification

`demostrar` introduces a property that the compiler should analyze.

Evidence states include:

- DEMONSTRATED;
- CHECKED;
- MEASURED;
- SIMULATED;
- OBSERVED;
- ASSUMED;
- UNKNOWN;
- REFUTED;
- NOT ANALYZED.

Tests are execution evidence. They are not silently promoted to mathematical proof.

Contracts may cover:

- inputs;
- outputs;
- invariants;
- resource states;
- capabilities;
- effects;
- memory;
- timing;
- energy;
- distribution;
- determinism;
- security.

Counterexamples should be reported whenever possible.

Verification evidence should retain relevant hypotheses, compiler version, dependencies, target architecture, and verification method.

## 16. Metaprogramming and reflection

Code, types, schemas, contracts, and modules are inspectable entities.

Capabilities include:

- type inspection;
- module inspection;
- contract inspection;
- AST transformation;
- code generation;
- schema-to-code generation;
- binding generation;
- documentation generation.

Metaprogramming uses the same effect, capability, resource, and restriction model as ordinary code.

## 17. Interoperability and ABI

AXIOM will define a language-owned ABI.

The ABI covers:

- value representation;
- layout;
- alignment;
- calling conventions;
- structures;
- errors;
- resources;
- capabilities;
- versions;
- binary compatibility.

Interop targets include C, C++, Rust, Python, .NET, WebAssembly, native platform APIs, and hardware interfaces.

WebAssembly is a target and interoperability boundary, not the foundation of AXIOM.

## 18. Language evolution

AXIOM will define:

- language versions;
- source compatibility;
- semantic compatibility;
- package compatibility;
- ABI compatibility;
- deprecations;
- migration rules.

The intended migration tool is `axiom migrate`.

A migration should:

1. analyze the project;
2. explain the change;
3. prepare a diff;
4. recompile;
5. verify affected behavior;
6. require explicit acceptance before changing source.

## 19. Ecosystem

The official standard library is intended to cover:

- text;
- collections;
- files;
- processes;
- system interfaces;
- networking;
- HTTP/WebSocket/TCP/UDP/DNS/TLS;
- serialization;
- JSON/XML/YAML/CSV/binary formats;
- time;
- mathematics;
- statistics;
- linear algebra;
- simulation;
- cryptography;
- authentication and certificates;
- databases and SQL;
- compression;
- image/audio/video codecs;
- UI;
- 2D/3D graphics;
- GPU and shaders;
- AI/ML and inference;
- testing;
- benchmarking;
- fuzzing;
- observability;
- FFI;
- devices;
- hardware;
- SIMD;
- embedded systems.

The intended toolchain includes:

```text
new
check
build
run
test
package
add
install
native-build
explain
format
lint
migrate
```

Build identity is:

```text
project + dependencies + compiler + configuration + target
```

Supply-chain metadata should include origin, identity, version, integrity, signature, dependencies, capabilities, effects, supported platforms, ABI information, and reproducibility data.

## 20. Modes

Modes are optional, combinable, and extensible.

Examples:

- systems;
- embedded;
- real_time;
- scientific;
- high_performance;
- secure;
- critical;
- distributed;
- hardware;
- GPU;
- web;
- mobile;
- quantum.

A mode changes priorities, constraints, guarantees, and compilation strategies. It does not create a different language.

## 21. Explainability

The compiler should explain:

- what a program does;
- which resources it uses;
- which capabilities it requires;
- which effects it produces;
- what parallelism was discovered;
- where data is placed;
- which optimizations were applied;
- which properties were demonstrated;
- what could not be demonstrated;
- why a CPU/GPU/NPU/QPU target was selected;
- which restrictions prevented alternatives.

Explainability is a compiler feature. It does not require an external AI service.

## 22. Hardware and low-level access

AXIOM must support the complete range from high-level applications to direct systems and hardware programming.

Conceptually:

```text
application
    ↓
system
    ↓
hardware
    ↓
memory / CPU / GPU / registers / instructions / interrupts
```

High-level abstractions are allowed and encouraged, but they must not create an artificial ceiling that prevents legitimate low-level programming.

## 23. Identity audit

AXIOM must not become a superficial variation of another language.

The following are not foundational pillars:

- `fn`/`def`/`function`;
- separate class/struct language systems;
- ownership/borrow/lifetime syntax;
- async/await as the concurrency model;
- try/catch as the failure architecture;
- import/include as textual inclusion;
- a fixed if/for/match taxonomy as the semantic foundation;
- an `unsafe` world separate from the rest of the language.

AXIOM's identity is:

```text
ENTITY
+
RELATION
+
INTENTION
+
RESOURCE
+
CAPABILITY
+
EFFECT
+
RESTRICTION
+
KNOWLEDGE
+
PLAN
```

## 24. Implementation authority

This specification is the architectural reference for AXIOM 0.1.

When the historical prototype conflicts with this specification, implementation is migrated toward the specification.

The specification itself should only change when the language design intentionally evolves and the change is recorded through the versioning process.

The current canonical syntax becomes stable through executable conformance tests, not by preserving accidental behavior of the prototype.
