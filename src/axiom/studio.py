"""Local AXIOM Studio workspace and language-service primitives."""

from __future__ import annotations

import argparse
import io
import json
import re
import subprocess
import zipfile
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import urlparse

from .ast import Function, Program
from .lexer import LexError, lex
from .parser import ParseError, parse
from .ir import lower
from .runtime import execute
from .semantic import SemanticError, analyze

_KEYWORDS = {
    "fn",
    "let",
    "return",
    "if",
    "else",
    "while",
    "for",
    "break",
    "continue",
    "struct",
    "true",
    "false",
}
_TYPES = {"Int", "Float", "Bool", "String"}
_BUILTINS = {"print", "len", "abs", "min", "max"}


def _diagnostic(error: Exception) -> dict[str, str]:
    message = str(error)
    line = "1"
    column = "1"
    if " at " in message:
        message, location = message.rsplit(" at ", 1)
        if ":" in location:
            line, column = location.split(":", 1)
    return {"severity": "error", "message": message, "line": line, "column": column}


def check_source(source: str) -> dict[str, object]:
    try:
        program = parse(source)
        analyze(program)
    except (LexError, ParseError, SemanticError) as error:
        return {"ok": False, "diagnostics": [_diagnostic(error)]}
    return {"ok": True, "diagnostics": [], "functions": [function.name for function in program.functions]}


def _function_names(program: Program) -> set[str]:
    return {function.name for function in program.functions if isinstance(function, Function)}


def complete_source(source: str, prefix: str = "") -> list[str]:
    candidates = sorted(_KEYWORDS | _TYPES | _BUILTINS)
    try:
        program = parse(source)
    except (LexError, ParseError):
        program = None
    if program is not None:
        candidates.extend(_function_names(program))
        candidates.extend(struct.name for struct in program.structs)
    else:
        candidates.extend(re.findall(r"\bfn\s+([A-Za-z_][A-Za-z0-9_]*)", source))
        candidates.extend(re.findall(r"\bstruct\s+([A-Za-z_][A-Za-z0-9_]*)", source))
    return sorted({item for item in candidates if item.startswith(prefix)})



def run_source(source: str) -> dict[str, object]:
    output = io.StringIO()
    try:
        program = parse(source)
        analyze(program)
        execute(lower(program), emit=lambda value: output.write(value + "\n"))
    except (LexError, ParseError, SemanticError, ValueError) as error:
        return {"ok": False, "diagnostics": [_diagnostic(error)], "output": output.getvalue()}
    return {"ok": True, "diagnostics": [], "output": output.getvalue()}


def run_workspace_tests(root: Path) -> dict[str, object]:
    failures: list[dict[str, str]] = []
    sources = sorted(path for path in root.rglob("*.ax") if ".git" not in path.parts)
    for path in sources:
        try:
            program = parse(path.read_text(encoding="utf-8"))
            analyze(program)
            execute(lower(program), emit=lambda _: None)
        except (OSError, LexError, ParseError, SemanticError, ValueError) as error:
            failures.append({"path": path.relative_to(root).as_posix(), **_diagnostic(error)})
    return {"ok": not failures, "count": len(sources), "failures": failures}


def git_status(root: Path) -> dict[str, object]:
    try:
        result = subprocess.run(
            ["git", "status", "--short", "--branch"],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return {"ok": False, "error": str(error)}
    return {"ok": result.returncode == 0, "output": result.stdout, "error": result.stderr}


def package_workspace(root: Path, output: Path | None = None) -> dict[str, object]:
    manifest = root / "axiom.toml"
    if not manifest.exists():
        return {"ok": False, "error": "axiom.toml not found"}
    target = output or root.with_suffix(".axpkg")
    try:
        with zipfile.ZipFile(target, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for path in root.rglob("*"):
                relative = path.relative_to(root)
                if not path.is_file():
                    continue
                if target.parent == root and relative == target.relative_to(root):
                    continue
                if any(part in {".git", "__pycache__", "target"} for part in relative.parts) or path.suffix == ".axpkg":
                    continue
                archive.write(path, relative.as_posix())
    except OSError as error:
        return {"ok": False, "error": str(error)}
    return {"ok": True, "output": str(target)}

def workspace_info(root: Path) -> dict[str, object]:
    root = root.resolve()
    manifest = root / "axiom.toml"
    sources = sorted(path.relative_to(root).as_posix() for path in root.rglob("*.ax") if ".git" not in path.parts)
    return {
        "root": str(root),
        "manifest": manifest.exists(),
        "sources": sources,
    }

_INDEX_HTML = """<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>AXIOM Studio</title>
<style>
:root { color-scheme: dark; font-family: system-ui, sans-serif; }
body { margin: 0; background: #111; color: #eee; }
header { padding: 12px 18px; border-bottom: 1px solid #333; display: flex; justify-content: space-between; }
main { display: grid; grid-template-columns: 220px 1fr 320px; height: calc(100vh - 53px); }
aside, section { border-right: 1px solid #333; overflow: auto; }
#files { padding: 12px; }
#files button { display: block; width: 100%; text-align: left; margin: 2px 0; }
textarea { width: 100%; height: 100%; box-sizing: border-box; resize: none; border: 0; outline: 0; padding: 18px; background: #181818; color: #f5f5f5; font: 14px/1.5 monospace; }
button { background: #222; color: #eee; border: 1px solid #444; padding: 6px 10px; cursor: pointer; }
#diagnostics { padding: 12px; white-space: pre-wrap; font-family: monospace; }
.ok { color: #9f9; }
.error { color: #f88; }
</style>
</head>
<body>
<header><strong>AXIOM Studio</strong><span><button onclick="runCode()">Run</button> <button onclick="runTests()">Test</button> <button onclick="showGit()">Git</button> <button onclick="packageProject()">Package</button> <span id="status">Ready</span></span></header>
<main>
<aside id="files"></aside>
<section><textarea id="editor" spellcheck="false"></textarea></section>
<aside id="diagnostics"></aside>
</main>
<script>
const editor = document.getElementById('editor');
const files = document.getElementById('files');
const diagnostics = document.getElementById('diagnostics');
const status = document.getElementById('status');
let sourcePath = null;
async function workspace() {
  const data = await fetch('/api/workspace').then(r => r.json());
  files.innerHTML = '';
  for (const path of data.sources) {
    const button = document.createElement('button');
    button.textContent = path;
    button.onclick = () => loadFile(path);
    files.appendChild(button);
  }
  if (data.sources.length) loadFile(data.sources[0]);
}
async function loadFile(path) {
  sourcePath = path;
  const data = await fetch('/api/source?path=' + encodeURIComponent(path)).then(r => r.json());
  editor.value = data.source;
  status.textContent = path;
  check();
}
async function runCode() { const data = await fetch('/api/run', {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({source: editor.value})}).then(r => r.json()); diagnostics.textContent = data.ok ? data.output || 'Program completed with no output.' : data.diagnostics.map(d => d.line + ':' + d.column + ' ' + d.message).join('\n'); }
async function runTests() { const data = await fetch('/api/test', {method:'POST'}).then(r => r.json()); diagnostics.textContent = data.ok ? 'Tests passed: ' + data.count + ' source file(s)' : data.failures.map(d => d.path + ' ' + d.line + ':' + d.column + ' ' + d.message).join('\n'); }
async function showGit() { const data = await fetch('/api/git').then(r => r.json()); diagnostics.textContent = data.output || data.error || 'clean'; }
async function packageProject() { const data = await fetch('/api/package', {method:'POST'}).then(r => r.json()); diagnostics.textContent = data.ok ? 'Packaged: ' + data.output : data.error; }
async function check() {
  const data = await fetch('/api/check', {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({source: editor.value})}).then(r => r.json());
  diagnostics.innerHTML = data.ok ? '<div class="ok">No diagnostics</div>' : data.diagnostics.map(d => '<div class="error">' + d.line + ':' + d.column + ' ' + d.message + '</div>').join('');
}
editor.addEventListener('input', () => { status.textContent = sourcePath ? sourcePath + ' *' : 'Untitled'; });
editor.addEventListener('keydown', async event => {
  if (event.key === ' ' && event.ctrlKey) {
    event.preventDefault();
    const before = editor.value.slice(0, editor.selectionStart);
    const match = before.match(/[A-Za-z_][A-Za-z0-9_]*$/);
    const prefix = match ? match[0] : '';
    const data = await fetch('/api/complete', {method:'POST', headers:{'content-type':'application/json'}, body:JSON.stringify({source: editor.value, prefix})}).then(r => r.json());
    status.textContent = data.items.length ? data.items.join('  ') : 'No completions';
    return;
  }
  if (event.key === 's' && (event.ctrlKey || event.metaKey)) { event.preventDefault(); check(); }
});
workspace();
</script>
</body>
</html>"""


class StudioHandler(BaseHTTPRequestHandler):
    root = Path.cwd()

    def _send_json(self, payload: dict[str, object], status: int = 200) -> None:
        body = json.dumps(payload).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self) -> None:
        parsed = urlparse(self.path)
        if parsed.path == "/":
            body = _INDEX_HTML.encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
            return
        if parsed.path == "/api/workspace":
            self._send_json(workspace_info(self.root))
            return
        if parsed.path == "/api/git":
            self._send_json(git_status(self.root))
            return
        if parsed.path == "/api/source":
            from urllib.parse import parse_qs
            relative = parse_qs(parsed.query).get("path", [""])[0]
            path = (self.root / relative).resolve()
            if self.root not in path.parents or path.suffix != ".ax":
                self._send_json({"error": "invalid source path"}, 400)
                return
            try:
                source = path.read_text(encoding="utf-8")
            except OSError as error:
                self._send_json({"error": str(error)}, 404)
                return
            self._send_json({"path": relative, "source": source})
            return
        self._send_json({"error": "not found"}, 404)

    def do_POST(self) -> None:
        endpoint = urlparse(self.path).path
        if endpoint not in {"/api/check", "/api/complete", "/api/run", "/api/test", "/api/package"}:
            self._send_json({"error": "not found"}, 404)
            return
        if endpoint in {"/api/test", "/api/package"}:
            if endpoint == "/api/test":
                self._send_json(run_workspace_tests(self.root))
            else:
                self._send_json(package_workspace(self.root))
            return
        try:
            length = int(self.headers.get("Content-Length", "0"))
            payload = json.loads(self.rfile.read(length).decode("utf-8"))
            source = payload["source"]
            if not isinstance(source, str):
                raise ValueError("source must be a string")
        except (ValueError, KeyError, json.JSONDecodeError) as error:
            self._send_json({"error": str(error)}, 400)
            return
        if endpoint == "/api/check":
            self._send_json(check_source(source))
            return
        if endpoint == "/api/run":
            self._send_json(run_source(source))
            return
        prefix = payload.get("prefix", "")
        if not isinstance(prefix, str):
            self._send_json({"error": "prefix must be a string"}, 400)
            return
        self._send_json({"items": complete_source(source, prefix)})

    def log_message(self, format: str, *args) -> None:
        return


def serve(root: Path, host: str, port: int) -> None:
    StudioHandler.root = root.resolve()
    server = ThreadingHTTPServer((host, port), StudioHandler)
    print(f"AXIOM Studio: http://{host}:{port}")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(prog="axiom studio", description="run AXIOM Studio")
    parser.add_argument("path", type=Path, nargs="?", default=Path("."))
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8765)
    return parser
