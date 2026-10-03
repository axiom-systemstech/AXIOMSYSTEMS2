# AXIOM Conformance

## Purpose

Conformance tests are the executable contract between the AXIOM specification and its implementation.

A feature is not considered stable because it is documented. It is stable when its intended semantics are exercised by executable tests.

## Test layers

### Syntax conformance

Tests verify:

- tokenization;
- indentation;
- canonical keywords;
- expressions;
- definitions;
- calls;
- blocks;
- source locations.

### Semantic conformance

Tests verify:

- types;
- forms and entities;
- name resolution;
- module resolution;
- relations;
- capabilities;
- effects;
- Resource Flow;
- Information Flow;
- restrictions;
- contracts.

### Execution conformance

Tests verify observable behavior of valid programs.

Where behavior is target-dependent, the test must declare the relevant target and constraints.

### Diagnostic conformance

Diagnostics are part of the language contract.

Tests should verify:

- source location;
- error category;
- explanation;
- relevant context;
- ambiguity handling;
- cascading-error suppression where appropriate.

Exact wording should only be asserted when wording itself is a deliberate interface.

### Determinism conformance

Identical declared inputs should produce identical compiler outputs wherever deterministic output is part of the contract.

Self-hosting tests must record:

- compiler revision;
- source revision;
- dependencies;
- target;
- generated artifact identity;
- comparison result.

### Bootstrap conformance

Bootstrap tests verify that a new compiler implementation can reproduce the required compiler artifacts using the previous bootstrap boundary.

Bootstrap is a transition mechanism, not the final language specification.

## Canonical examples

A minimal canonical program is:

```axiom
show("Hello, AXIOM")
```

A canonical entity definition is:

```axiom
user:
    name: String
    age: Int
```

A canonical entity instance is:

```axiom
alex: user
    name = "Alex"
    age = 25
```

These examples are executable conformance fixtures in tests/test_canonical.py.

## Historical compatibility tests

The repository currently contains tests for the historical prototype syntax.

Those tests remain useful during migration, but they must not prevent the canonical AXIOM 0.1 implementation from evolving.

When a historical feature is intentionally retired:

1. mark the compatibility test as transitional;
2. add the canonical replacement test;
3. document the migration;
4. remove the obsolete test only after the replacement is verified.

## Completion gate

A semantic feature is complete only when:

```text
specification
    =
implementation
    =
tests
    =
diagnostics
    =
documentation
```

Any unresolved difference remains an implementation task.
