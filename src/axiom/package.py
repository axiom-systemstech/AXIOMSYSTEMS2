from __future__ import annotations

import json
import re
import shutil
import tomllib
from dataclasses import dataclass
from pathlib import Path

_PACKAGE_NAME = re.compile(r"^[a-z][a-z0-9_-]*$")


class PackageError(ValueError):
    pass


@dataclass(frozen=True)
class Package:
    name: str
    version: str
    root: Path


def _read_manifest(path: Path) -> Package:
    try:
        data = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        raise PackageError(f"invalid package manifest: {error}") from error
    package = data.get("package")
    if not isinstance(package, dict):
        raise PackageError("package manifest requires [package]")
    name = package.get("name")
    version = package.get("version")
    if not isinstance(name, str) or not _PACKAGE_NAME.fullmatch(name):
        raise PackageError("package name is invalid")
    if not isinstance(version, str) or not version:
        raise PackageError("package version is invalid")
    return Package(name, version, path.parent)


def _write_project_manifest(path: Path, project_name: str, dependencies: dict[str, str]) -> None:
    lines = [
        "[package]",
        f'name = "{project_name}"',
        'version = "0.1.0"',
        "",
        "[dependencies]",
    ]
    lines.extend(f'{name} = "{version}"' for name, version in sorted(dependencies.items()))
    path.write_text(chr(10).join(lines) + chr(10), encoding="utf-8")


def add_package(project_root: Path, name: str, registry_root: Path) -> Package:
    if not _PACKAGE_NAME.fullmatch(name):
        raise PackageError(f"invalid package name '{name}'")

    project_manifest = project_root / "axiom.toml"
    if project_manifest.exists():
        project = tomllib.loads(project_manifest.read_text(encoding="utf-8"))
        project_data = project.get("package", {})
        project_name = project_data.get("name", project_root.name)
        dependencies = dict(project.get("dependencies", {}))
    else:
        project_name = project_root.name
        dependencies = {}
        _write_project_manifest(project_manifest, project_name, dependencies)

    package_manifest = registry_root / name / "axiom.toml"
    package = _read_manifest(package_manifest)

    vendor_root = project_root / "vendor"
    destination = vendor_root / package.name
    if destination.exists():
        shutil.rmtree(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copytree(package.root, destination)

    dependencies[package.name] = package.version
    _write_project_manifest(project_manifest, project_name, dependencies)

    lock = {
        "version": 1,
        "packages": {
            dependency: {"version": version, "source": "local-registry"}
            for dependency, version in sorted(dependencies.items())
        },
    }
    (project_root / "axiom.lock").write_text(
        json.dumps(lock, indent=2, sort_keys=True) + chr(10),
        encoding="utf-8",
    )
    return package
