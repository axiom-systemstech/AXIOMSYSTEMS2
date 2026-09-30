# AXIOM Package Manager

The package manager is introduced with a local registry first. This keeps dependency resolution deterministic while the registry protocol is still being designed.

## Project manifest

An AXIOM project uses axiom.toml:

    [package]
    name = "example"
    version = "0.1.0"

    [dependencies]
    networking = "0.1.0"

The dependency table records the exact version selected for the project.

## Adding a package

The CLI command is:

    axiom add networking

The default registry is the registry/ directory in the current working directory. A registry can be selected explicitly:

    axiom add networking --registry /path/to/registry

A registry package is a directory containing its own axiom.toml. The package is copied into vendor/<name>, the project manifest is updated, and axiom.lock records the resolved package version and source.

## Lock file

axiom.lock is JSON for the bootstrap implementation. It contains a format version and one entry per resolved dependency.

The lock file is generated from the resolved manifest and local registry metadata. It is intended to become the compatibility boundary for future remote registry resolution.

## Registry direction

The local registry is deliberately a narrow first implementation. Future work can replace the source resolver without changing the project dependency model.

Remote publishing, authentication, package archives, integrity hashes and registry discovery belong to later iterations of AXIOM PKG.
