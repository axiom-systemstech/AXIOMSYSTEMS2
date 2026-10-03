"""Canonical AXIOM 0.1 frontend migration layer."""
from __future__ import annotations
from .ast import Call, Function, Program

class NewSyntaxError(ValueError):
    pass

def _translate(text: str) -> str:
    if text.startswith("mostrar(") and text.endswith(")"):
        return "print(" + text[len("mostrar("):]
    return text

def parse_new(source: str) -> Program:
    from .parser import parse as parse_legacy
    lines = [line.strip() for line in source.splitlines() if line.strip()]
    if not lines:
        raise NewSyntaxError("programa vacío")
    body = []
    names = set()
    for number, text in enumerate(lines, 1):
        if text.startswith(("decidir ", "repetir ", "cuando ", "usar ", "ofrecer ", "necesita:", "puede:", "prefiere:", "restringir:", "demostrar:", "modo:")):
            raise NewSyntaxError(f"construcción todavía no conectada al pipeline en línea {number}")
        text = _translate(text)
        if "=" in text and not any(op in text for op in ("==", ">=", "<=")):
            name, value = text.split("=", 1)
            name = name.strip()
            if not name.isidentifier():
                raise NewSyntaxError(f"identificador inválido en línea {number}")
            prefix = "let " if name not in names else ""
            if prefix:
                names.add(name)
            text = prefix + name + " = " + value.strip()
        parsed = parse_legacy("fn main() { " + text + " }")
        statements = parsed.functions[0].body
        if len(statements) != 1:
            raise NewSyntaxError(f"expresión inválida en línea {number}")
        statement = statements[0]
        if not isinstance(statement, (Call,)) and not text.startswith("let "):
            raise NewSyntaxError(f"se esperaba llamada o asignación en línea {number}")
        body.append(statement)
    return Program([Function("main", body)])

def looks_like_new_syntax(source: str) -> bool:
    lines = [line.strip() for line in source.splitlines() if line.strip()]
    return bool(lines) and not any(line.startswith(("fn ", "module ", "import ", "struct ", "let ", "if ", "while ", "for ")) for line in lines)
