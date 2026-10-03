"""Public compiler-core API for project-wide AXIOM compilation."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .ast import Program
from .ir import IRProgram, lower
from .parser import parse
from .semantic import analyze
from .semantic_model import SemanticModel

class CompilerError(ValueError):
    """Raised for project/module graph errors."""

@dataclass(frozen=True)
class SourceUnit:
    module: str
    path: Path
    source: str
    program: Program

@dataclass(frozen=True)
class Compilation:
    program: Program
    ir: IRProgram
    semantic: SemanticModel
    units: tuple[SourceUnit, ...]

class ModuleResolver:
    """Resolve AXIOM modules from a source tree without leaking filesystem details."""

    def __init__(self, root: Path):
        self.root = root.resolve()
        self.source_root = self.root / "src" if (self.root / "src").is_dir() else self.root
        self._units: dict[str, SourceUnit] = {}
        self._loading: set[str] = set()

    def resolve(self, entry: Path | None = None) -> tuple[SourceUnit, ...]:
        entry = (entry or self._default_entry()).resolve()
        if not entry.is_file():
            raise CompilerError(f"source file not found: {entry}")
        entry_module = self._module_for_path(entry)
        self._visit(entry_module, entry)
        return tuple(self._units[name] for name in sorted(self._units))

    def _default_entry(self) -> Path:
        candidates = [self.source_root / "main.ax", self.source_root / "main" / "main.ax"]
        for candidate in candidates:
            if candidate.is_file():
                return candidate
        files = sorted(self.source_root.rglob("*.ax"))
        if len(files) == 1:
            return files[0]
        raise CompilerError("project has no unique entry source")

    def _module_for_path(self, path: Path) -> str:
        relative = path.relative_to(self.source_root).with_suffix("")
        return ".".join(relative.parts)

    def _path_for_module(self, module: str) -> Path:
        path = self.source_root.joinpath(*module.split(".")).with_suffix(".ax")
        if not path.is_file():
            raise CompilerError(f"module '{module}' not found")
        return path

    def _visit(self, module: str, path: Path) -> None:
        if module in self._units:
            return
        if module in self._loading:
            raise CompilerError(f"cyclic module import involving '{module}'")
        self._loading.add(module)
        program = parse(path.read_text(encoding="utf-8"))
        declared = program.module_name or module
        if declared != module:
            raise CompilerError(f"module declaration '{declared}' does not match path module '{module}'")
        unit = SourceUnit(module, path, path.read_text(encoding="utf-8"), program)
        for imported in program.imports:
            self._visit(imported, self._path_for_module(imported))
        self._loading.remove(module)
        self._units[module] = unit

class Compiler:
    """Stable frontend → semantic analysis → IR compiler interface."""

    def compile_project(self, root: str | Path, entry: str | Path | None = None) -> Compilation:
        resolver = ModuleResolver(Path(root))
        units = resolver.resolve(Path(entry) if entry is not None else None)
        functions = []
        structs = []
        for unit in units:
            functions.extend(unit.program.functions)
            structs.extend(unit.program.structs)
        program = Program(functions, structs, canonical=all(unit.program.canonical for unit in units))
        semantic = analyze(program)
        return Compilation(program, lower(program), semantic, units)

    def compile_source(self, source: str) -> Compilation:
        program = parse(source)
        semantic = analyze(program)
        return Compilation(program, lower(program), semantic, ())
