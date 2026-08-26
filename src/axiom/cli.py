"""Command-line entry point for the AXIOM developer tool."""

from __future__ import annotations

import argparse
import platform
import sys
from pathlib import Path

from . import __version__
from .ir import lower
from .parser import parse
from .runtime import execute_program
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
    if args.command == "run":
        try:
            program = parse(args.source.read_text(encoding="utf-8"))
            analyze(program)
            execute_program(program)
        except (OSError, ValueError) as error:
            print(f"error: {error}", file=sys.stderr)
            return 1
        return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())