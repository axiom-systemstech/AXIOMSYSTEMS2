# AXIOM Standard Library

The standard library is introduced in layers. The first layer is axiom.std, implemented as compiler-known built-ins so the language can provide useful core operations before the module and package systems are introduced.

## Core functions

| Function | Signature | Description |
| --- | --- | --- |
| len | len(String) -> Int | Returns the number of Unicode scalar values in a string. |
| len | len(T[]) -> Int | Returns the number of elements in an array. |
| abs | abs(Int) -> Int | Returns the absolute value of an integer. |
| abs | abs(Float) -> Float | Returns the absolute value of a floating-point value. |
| min | min(Int, Int) -> Int | Returns the smaller integer. |
| min | min(Float, Float) -> Float | Returns the smaller floating-point value. |
| max | max(Int, Int) -> Int | Returns the larger integer. |
| max | max(Float, Float) -> Float | Returns the larger floating-point value. |

These functions are part of the language's standard environment and do not require a user-defined function declaration.

## Implementation boundary

The Python frontend/runtime and native Rust compiler/VM expose the same signatures and runtime behavior.

The remaining standard-library families are introduced after the core layer:

- axiom.io
- axiom.fs
- axiom.net
- axiom.http
- axiom.crypto
- axiom.math
- axiom.time
- axiom.collections
- axiom.concurrent
- axiom.process
- axiom.system

Module namespaces and imports are intentionally deferred until the language grammar and package model are ready to support them cleanly.
