"""Command-line entry point for the AXIOM developer tool."""

from __future__ import annotations

import argparse
import platform
import sys
import zipfile
from pathlib import Path

from . import __version__
from .ir import lower
from .package import PackageError, add_package
from .parser import parse
from .runtime import execute
from .semantic import analyze


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="axiom", description="AXIOM SYSTEMS developer tool")
    parser.add_argument("--version", action="version", version=f"axiom {__version__}")
    subparsers = parser.add_subparsers(dest="command")
    subparsers.add_parser("doctor", help="check the local AXIOM bootstrap environment")
    check_parser = subparsers.add_parser("check", help="parse an AXIOM source file")
    check_parser.add_argument("source", type=Path)
    build_parser = subparsers.add_parser("build", help="compile an AXIOM source file to AXIOM-IR")
    build_parser.add_argument("source", type=Path)
    build_parser.add_argument("-o", "--output", type=Path)
    run_parser = subparsers.add_parser("run", help="compile and execute an AXIOM source file")
    run_parser.add_argument("source", type=Path)
    new_parser = subparsers.add_parser("new", help="create an AXIOM project")
    new_parser.add_argument("name", type=Path)
    test_parser = subparsers.add_parser("test", help="run AXIOM source tests")
    test_parser.add_argument("path", type=Path, nargs="?", default=Path("tests"))
    package_parser = subparsers.add_parser("package", help="create a distributable AXIOM package")
    package_parser.add_argument("-o", "--output", type=Path)
    add_parser = subparsers.add_parser("add", help="add a package from the local AXIOM registry")
    add_parser.add_argument("name")
    add_parser.add_argument("--registry", type=Path)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.command == "doctor":
        print(f"axiom {__version__}")
        print(f"python {platform.python_version()}")
        print(f"platform {sys.platform}")
        return 0
    if args.command == "check":
        try:
            analyze(parse(args.source.read_text(encoding="utf-8")))
        except (OSError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"ok: {args.source}")
        return 0
    if args.command == "build":
        try:
            program = parse(args.source.read_text(encoding="utf-8"))
            analyze(program)
            output = args.output or args.source.with_suffix(".air")
            output.write_text(lower(program).render(), encoding="utf-8")
        except (OSError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"built: {output}")
        return 0
    if args.command == "new":
        try:
            project = args.name
            if project.exists():
                raise ValueError(f"project path already exists: {project}")
            (project / "src").mkdir(parents=True)
            (project / "tests").mkdir()
            (project / "axiom.toml").write_text(
                chr(10).join([
                    "[package]",
                    f'name = "{project.name}"',
                    'version = "0.1.0"',
                    "",
                    "[dependencies]",
                    "",
                ]),
                encoding="utf-8",
            )
            (project / "src" / "main.ax").write_text(
                'fn main() { print("Hello AXIOM") }' + chr(10),
                encoding="utf-8",
            )
        except OSError as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"created: {project}")
        return 0
    if args.command == "test":
        if not args.path.exists():
            print(f"error: test path does not exist: {args.path}", file=sys.stderr)
            return 1
        sources = sorted(args.path.rglob("*.ax")) if args.path.is_dir() else [args.path]
        try:
            for source in sources:
                program = parse(source.read_text(encoding="utf-8"))
                analyze(program)
                execute(lower(program))
        except (OSError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"test ok: {len(sources)} source file(s)")
        return 0
    if args.command == "package":
        root = Path.cwd()
        manifest = root / "axiom.toml"
        if not manifest.exists():
            print("error: axiom.toml not found", file=sys.stderr)
            return 1
        output = args.output or root.with_suffix(".axpkg")
        try:
            with zipfile.ZipFile(output, "w", compression=zipfile.ZIP_DEFLATED) as archive:
                for path in root.rglob("*"):
                    relative = path.relative_to(root)
                    if (
                        not path.is_file()
                        or relative == output.relative_to(root)
                        or any(part in {".git", "__pycache__", "target"} for part in relative.parts)
                        or path.suffix == ".axpkg"
                    ):
                        continue
                    archive.write(path, relative.as_posix())
        except OSError as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"packaged: {output}")
        return 0
    if args.command == "add":
        try:
            registry = args.registry or Path("registry")
            package = add_package(Path.cwd(), args.name, registry)
        except (OSError, PackageError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        print(f"added: {package.name} {package.version}")
        return 0
    if args.command == "run":
        try:
            program = parse(args.source.read_text(encoding="utf-8"))
            analyze(program)
            execute(lower(program))
        except (OSError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())