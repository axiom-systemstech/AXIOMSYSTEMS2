# AXIOM Version Roadmap

This roadmap defines the progressive implementation path from the current transition infrastructure to AXIOM 1.0.

Version numbers describe engineering maturity and conformance gates. They are not calendar commitments.

## 0.1 — Canonical foundation

**Purpose:** turn the closed conceptual design into an executable language foundation.

Required:

- canonical lexer;
- canonical parser;
- canonical AST/entity representation;
- core types and units;
- name and module resolution;
- semantic diagnostics;
- initial Resource Flow;
- initial effects and capabilities;
- executable conformance suite.

## 0.2 — Semantic compiler

**Purpose:** make AXIOM useful for non-trivial programs.

Required:

- complete core expressions;
- blocks as values;
- decisions and repetition;
- functions and entities;
- modules and packages;
- Resource Flow analysis;
- Information Flow foundations;
- effect/capability propagation;
- restrictions and preferences;
- contracts;
- deterministic AXIOM IR.

## 0.3 — Self-hosted compiler

**Purpose:** AXIOM becomes the primary implementation language of its compiler.

Required:

- lexer in AXIOM;
- parser in AXIOM;
- AST model in AXIOM;
- semantic compiler in AXIOM;
- IR generation in AXIOM;
- compiler driver in AXIOM;
- compiler self-build;
- reproducible self-build.

## 0.4 — Native foundation

**Purpose:** move execution ownership from the historical host toward AXIOM.

Required:

- AXIOM runtime interface;
- memory/resource execution model;
- native target abstraction;
- initial AXIOM ABI;
- low-level interop boundary;
- native bootstrap without Python.

## 0.5 — Toolchain ownership

**Purpose:** make the core developer workflow AXIOM-owned.

Required:

- compiler CLI;
- project workflow;
- package management;
- deterministic builds;
- formatter;
- linter;
- test runner;
- explainability tooling;
- migration tooling.

## 0.6–0.9 — Stabilization

Focus areas:

- concurrency and execution planning;
- real-time behavior;
- determinism;
- fault tolerance;
- distributed execution;
- metaprogramming;
- reflection;
- FFI;
- ABI compatibility;
- security and Information Flow;
- optimization;
- standard library;
- platform targets;
- ecosystem tooling.

Each feature receives a conformance test before it is considered stable.

## 1.0 — Independence and conformance

AXIOM 1.0 requires:

- canonical language specification implemented;
- compiler self-hosting;
- reproducible compiler self-build;
- no Python dependency in the core toolchain;
- no Rust dependency as the language implementation foundation;
- AXIOM-owned runtime/backend boundaries;
- stable ABI and FFI contracts;
- conformance suite;
- deterministic package/build identity;
- documented migration and compatibility policy;
- production-quality diagnostics;
- explainable compiler decisions.

## What 1.0 does not mean

AXIOM 1.0 does not mean every ecosystem feature is finished.

It means the language core and its implementation foundation are independent, stable, reproducible, and capable of supporting the broader AXIOM SYSTEMS roadmap.

Future capabilities such as advanced graphics, robotics, aircraft systems, industrial control, quantum targets, or AXIOM OS can then grow on top of an independent language foundation.
