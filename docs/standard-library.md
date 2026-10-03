# AXIOM Standard Library

The standard library will be an official semantic foundation of AXIOM rather than an unrelated collection of host-language helpers.

## Core direction

The library is expected to cover:

- text and Unicode;
- collections;
- files and storage;
- processes and system interfaces;
- networking;
- HTTP and WebSocket;
- serialization and data formats;
- time and clocks;
- mathematics and statistics;
- linear algebra and tensors;
- cryptography;
- databases and SQL;
- compression;
- images, audio, video, and codecs;
- UI;
- graphics and GPU;
- AI/ML and inference;
- testing, benchmarking, and fuzzing;
- observability;
- FFI and ABI;
- devices and hardware;
- embedded systems;
- simulation.

## Current transition layer

The historical implementation exposes a small set of compiler-known operations such as `len`, `abs`, `min`, and `max`.

These signatures are useful compatibility infrastructure, but they do not define the final standard library architecture.

## Design rule

Standard-library APIs should express semantic requirements and effects clearly.

A filesystem API should expose filesystem effects. A network API should expose network effects. A GPU API should expose device capabilities and resource requirements.

The compiler must be able to reason about these operations rather than treating the standard library as opaque host-language code.
