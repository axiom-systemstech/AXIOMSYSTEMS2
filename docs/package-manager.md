# AXIOM Package Manager

The package manager is being migrated from the historical local implementation toward the definitive AXIOM package model.

## Current project manifest

```toml
[package]
name = "example"
version = "0.1.0"

[dependencies]
networking = "0.1.0"
```

The current native transition implementation supports local registry packages, vendoring, and a deterministic lockfile.

## Package identity

The definitive model must include:

- package identity;
- version;
- origin;
- dependencies;
- integrity;
- signature;
- capabilities requested;
- effects;
- supported platforms;
- ABI information;
- reproducibility metadata.

## Security model

Installation should conceptually perform:

```text
resolve
  ↓
verify identity/integrity
  ↓
analyze capabilities/effects
  ↓
validate compatibility
  ↓
install
```

A dependency must not silently gain privileges merely because those privileges exist elsewhere in the package graph.

## Current limitation

Remote registries, cryptographic package verification, signatures, publishing, and full supply-chain policy are not yet the definitive AXIOM implementation.

The existing local registry workflow is transition infrastructure.
