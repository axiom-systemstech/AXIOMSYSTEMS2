"""AXIOM verification gates for conformance, bootstrap, independence, and reproducibility."""
from __future__ import annotations
import hashlib
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path

@dataclass(frozen=True)
class TestResult:
    name: str
    passed: bool
    detail: str

MODES = ("language", "conformance", "bootstrap", "independence", "reproducible")

def run_test_mode(mode: str, root: Path) -> tuple[TestResult, ...]:
    if mode == "language": return _pytest_gate(root, "language", "tests/test_canonical.py")
    if mode == "conformance": return _pytest_gate(root, "conformance", "tests/test_canonical.py", "tests/test_semantic_model.py")
    if mode == "bootstrap": return _bootstrap_gate(root)
    if mode == "independence": return _independence_gate(root)
    if mode == "reproducible": return _reproducibility_gate(root)
    raise ValueError(f"unknown AXIOM test mode '{mode}'")

def _pytest_gate(root: Path, name: str, *paths: str) -> tuple[TestResult, ...]:
    completed = subprocess.run([sys.executable, "-m", "pytest", "-q", *paths], cwd=root, capture_output=True, text=True)
    if completed.returncode == 0:
        lines = completed.stdout.strip().splitlines()
        return (TestResult(name, True, lines[-1] if lines else "tests passed"),)
    lines = (completed.stdout + completed.stderr).strip().splitlines()
    return (TestResult(name, False, lines[-1] if lines else "test suite failed"),)

def _bootstrap_gate(root: Path) -> tuple[TestResult, ...]:
    required = ("bootstrap/lexer.ax", "bootstrap/parser.ax", "bootstrap/ast.ax", "bootstrap/semantic.ax", "bootstrap/compiler.ax")
    missing = [path for path in required if not (root / path).is_file()]
    return (TestResult("bootstrap", not missing, "AXIOM-written bootstrap stages are present" if not missing else "missing: " + ", ".join(missing)),)

def _independence_gate(root: Path) -> tuple[TestResult, ...]:
    results = []
    manifest = root / "pyproject.toml"
    text = manifest.read_text(encoding="utf-8") if manifest.exists() else ""
    empty = "dependencies = []" in text
    results.append(TestResult("package-dependencies", empty, "no runtime Python packages declared" if empty else "runtime Python packages are declared"))
    python_host = bool(list((root / "src" / "axiom").glob("*.py")))
    results.append(TestResult("python-independence", not python_host, "Python compiler host remains" if python_host else "no Python compiler host found"))
    rust_sources = list((root / "native").rglob("*.rs")) if (root / "native").exists() else []
    results.append(TestResult("rust-independence", not rust_sources, "Rust native boundary remains" if rust_sources else "no Rust native boundary found"))
    bootstrap = (root / "bootstrap").is_dir()
    results.append(TestResult("axiom-bootstrap", bootstrap, "AXIOM bootstrap source is present" if bootstrap else "AXIOM bootstrap source is missing"))
    return tuple(results)

def _reproducibility_gate(root: Path) -> tuple[TestResult, ...]:
    source = root / "tests" / "fixtures" / "reproducibility.ax"
    if not source.exists(): return (TestResult("reproducible-build", False, "missing tests/fixtures/reproducibility.ax"),)
    outputs = (root / ".axiom-test-first.air", root / ".axiom-test-second.air")
    try:
        for output in outputs:
            completed = subprocess.run([sys.executable, "-m", "axiom.cli", "build", str(source), "--output", str(output)], cwd=root, capture_output=True, text=True)
            if completed.returncode: return (TestResult("reproducible-build", False, completed.stderr.strip() or "build failed"),)
        passed = _sha256(outputs[0]) == _sha256(outputs[1])
        return (TestResult("reproducible-build", passed, "identical artifacts" if passed else "artifact hashes differ"),)
    finally:
        for output in outputs: output.unlink(missing_ok=True)

def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def format_results(results: tuple[TestResult, ...]) -> str:
    lines = [f"[{'PASS' if item.passed else 'FAIL'}] {item.name}: {item.detail}" for item in results]
    passed = sum(item.passed for item in results)
    lines.append(f"AXIOM verification: {passed}/{len(results)} gates passed")
    return "\n".join(lines)
