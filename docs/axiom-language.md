# AXIOM Language

This document describes the canonical AXIOM 0.1 language direction. It replaces the historical prototype syntax as the design reference.

## Design principle

> Natural in expression, formal in meaning.

AXIOM keeps the visible syntax small while making the compiler semantically powerful. The language should be approachable on the first day without creating a separate "simple" and "advanced" language.

## Canonical syntax

AXIOM 0.1 uses English as its canonical human-facing syntax. The semantic core is independent of human language; future Spanish and other frontends can lower into the same semantic representation.

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

Blocks use four-space indentation. Braces are not normal block delimiters.

### Values and calls

```axiom
show("Hello, AXIOM")

name = "Alex"
age = 25
active = true
```

### Entities and forms

```axiom
user:
    name: String
    age: Int

alex: user
    name = "Alex"
    age = 25

show(alex.name)
```

A definition describes a form/type. An entity declaration instantiates or specializes that form.

### Functions

Functions do not require `fn`, `def`, or `function`.

```axiom
sum(a, b):
    a + b
```

A block normally produces the value of its final expression. Explicit return syntax is not the foundation of the language.

### Decisions

```axiom
if age >= 18:
    show("Adult")
else:
    show("Minor")
```

### Repetition

```axiom
repeat user in users:
    show(user.name)
```

The exact repetition grammar is being stabilized by executable conformance tests.

### Events

```axiom
when temperature > 90:
    stop(motor)
```

Time and periodic execution are semantic properties rather than a second asynchronous language.

### Dependencies and capabilities

```axiom
usar matematicas

necesita:
    filesystem.read
    network

puede:
    camera.read
```

`usar` declares semantic dependency. `necesita` declares a requirement. `puede` describes an allowed capability boundary.

### Preferences and restrictions

```axiom
prefiere:
    GPU
    energía = baja

restringir:
    dispositivo = GPU
    precisión = fp32
    ejecución = secuencial
```

Preferences guide planning. Restrictions are mandatory constraints.

### Contracts

```axiom
demostrar:
    temperatura < 100 °C
```

Verification results are classified by evidence strength. Tests are not silently promoted to mathematical proof.

## Types

AXIOM treats a type as semantic knowledge about an entity, including:

- meaning;
- structure;
- identity;
- capabilities;
- relations;
- valid states;
- restrictions;
- contracts;
- representation;
- verifiable knowledge.

Core types include:

```text
booleano
entero
real
decimal
texto
carácter
bytes
tiempo
duración
unidad
```

Scientific types include rational, complex, vector, matrix, tensor, interval, probability, and algebraic-number families.

Units and dimensionality are first-class semantic information:

```axiom
masa = 5 kg
velocidad = 9.81 m/s²
temperatura = 25 °C
periodo = 10 ms
```

The compiler must distinguish exact and approximate numerical operations.

There is no universal `null` model. Absence and failure are semantic possibilities that must be represented explicitly by the relevant type or contract.

## Resource Flow

AXIOM does not make ownership, borrowing, or lifetime syntax the foundation.

The compiler asks:

1. What must exist?
2. Who uses it?
3. What operations are performed?
4. What dependencies exist?
5. When is the last consumer finished?
6. Which representation satisfies the constraints at acceptable cost?

This applies to memory, files, sockets, GPU buffers, devices, processes, handles, connections, energy, time, and information.

The compiler may select copy, alias, view, move, sharing, local storage, GPU storage, or reuse when semantics permit it.

Explicit restrictions take priority.

## Effects, capabilities, and resources

These are three separate concepts:

```text
RESOURCE   = what an operation acts on
CAPABILITY = what it is allowed to do
EFFECT     = what it actually does
```

Effects are inferred and propagated through calls.

Examples include:

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

Capability provenance must remain explainable.

## Information Flow

Resource Flow and Information Flow are separate analyses.

A program may have permission to write to a network resource without being allowed to send every category of information through that resource.

Information classifications and policies can be domain-defined. The compiler must identify prohibited flows and show their provenance.

## Concurrency

Concurrency is an execution-plan property.

The compiler can identify independent operations, choose parallel execution, select CPU/SIMD/GPU/NPU/QPU/process/machine execution, and insert only the synchronization required by the semantic dependencies.

The language does not require `async/await`, `thread`, `mutex`, or `spawn` as foundational syntax.

Sequential execution remains expressible:

```axiom
restringir:
    ejecución = secuencial
```

## Time and real-time behavior

Time distinguishes:

- instant;
- duration;
- clock;
- period;
- deadline;
- latency;
- jitter;
- priority.

Hard guarantees are restrictions. Soft goals are preferences.

```axiom
modo:
    tiempo_real

restringir:
    periodo = 10 ms
    deadline = 5 ms
```

The compiler must report whether a timing requirement is proven, possible, not demonstrable, or impossible under known constraints.

## Fault tolerance

Failure is a normal semantic possibility.

Relevant actions include retry, timeout, fallback, restart, isolate, degrade, replace, propagate, recover, and cancel.

Retries must respect idempotency and effect contracts.

## Distribution

Local and distributed execution use the same semantic model.

The compiler and runtime must account for:

- nodes;
- network communication;
- serialization;
- latency;
- partitions;
- consistency;
- atomicity;
- idempotency;
- retries;
- timeouts;
- recovery.

RPC, queues, sockets, and shared memory are implementation mechanisms rather than separate language models.

## Modules and packages

The conceptual hierarchy is:

```text
PROJECT → PACKAGE → MODULE → SEMANTIC SPACE → ENTITY
```

`usar` expresses semantic dependency. `ofrecer` defines the public semantic interface.

Name resolution must be deterministic and must not search arbitrary project state merely to guess what the programmer meant.

## Explainability

The compiler should be able to explain:

- what a program does;
- resources it uses;
- capabilities it needs;
- effects it produces;
- concurrency decisions;
- memory placement;
- optimization decisions;
- proof/evidence status;
- target selection;
- restrictions that prevented alternatives.

This explanation is a compiler capability, not an external AI service.

## Compatibility with the historical prototype

The repository still contains the earlier `fn/struct/if` syntax because the historical implementation is being used as transition infrastructure.

That syntax is **not** the canonical AXIOM 0.1 language definition.

The canonical implementation is introduced incrementally and will be accepted through executable conformance tests before historical syntax is retired.
