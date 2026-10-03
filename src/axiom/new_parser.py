"""Canonical AXIOM 0.1 English frontend.

The canonical surface is indentation-based. This frontend lowers that stable
surface into the existing AST so the semantic and IR pipeline can validate it
while the implementation migrates toward a native AXIOM compiler.
"""

from __future__ import annotations

import re

from .ast import Program

class NewSyntaxError(ValueError):
    """Raised when canonical AXIOM syntax is invalid."""

_BLOCK_RE = re.compile(r"^(?P<body>.*?)[ ]*:$")
_IDENTIFIER = r"[A-Za-z_][A-Za-z0-9_]*"
_DIRECTIVES = ("use ", "provide ", "needs:", "can:", "prefer:", "restrict:", "prove:", "mode:")


def _indent(line: str, number: int) -> tuple[int, str]:
    if "\t" in line:
        raise NewSyntaxError(f"tabs are not allowed in canonical AXIOM at line {number}")
    stripped = line.lstrip(" ")
    if not stripped:
        return 0, ""
    width = len(line) - len(stripped)
    if width % 4:
        raise NewSyntaxError(f"indentation must use multiples of four spaces at line {number}")
    return width // 4, stripped


def _translate_statement(text: str, declared: set[str]) -> str:
    if text.startswith("show("):
        text = "print(" + text[len("show("):]
    assignment = re.match(rf"^({_IDENTIFIER})[ ]*=[ ]*(.+)$", text)
    if assignment and "==" not in text:
        name, value = assignment.groups()
        keyword = "let " if name not in declared else ""
        declared.add(name)
        return f"{keyword}{name} = {value}"
    return text


def _collect_child_lines(lines: list[tuple[int, str, int]], start: int, parent_depth: int) -> tuple[list[tuple[int, str, int]], int]:
    children: list[tuple[int, str, int]] = []
    index = start
    while index < len(lines) and lines[index][0] > parent_depth:
        children.append(lines[index])
        index += 1
    return children, index


def _canonical_to_legacy(source: str) -> str:
    lines: list[tuple[int, str, int]] = []
    for number, raw in enumerate(source.splitlines(), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        lines.append((*_indent(raw, number), number))

    if not lines:
        raise NewSyntaxError("empty AXIOM source")

    output: list[str] = []
    declared: set[str] = set()
    stack: list[tuple[int, str | None]] = []
    function_depths: list[int] = []
    index = 0

    def close_to(depth: int) -> None:
        while len(stack) > depth:
            block_depth, post = stack.pop()
            if post and post.startswith("@before:"):
                output.append(post[len("@before:"):])
            output.append("}")
            if function_depths and function_depths[-1] == block_depth:
                function_depths.pop()
            if post and not post.startswith("@before:"):
                output.append(post)

    while index < len(lines):
        depth, text, number = lines[index]
        if depth > len(stack):
            if depth != len(stack) + 1:
                raise NewSyntaxError(f"indentation jumps more than one level at line {number}")
        close_to(depth)

        typed_entity_header = re.match(rf"^({_IDENTIFIER})[ ]*:[ ]*({_IDENTIFIER})$", text)
        if (
            typed_entity_header
            and index + 1 < len(lines)
            and lines[index + 1][0] > depth
            and "=" in lines[index + 1][1]
        ):
            name, type_name = typed_entity_header.groups()
            children, next_index = _collect_child_lines(lines, index + 1, depth)
            if not children or any(child_depth != depth + 1 for child_depth, _, _ in children):
                raise NewSyntaxError(f"typed entity body must contain only fields at line {number}")
            fields = []
            for _, field_text, field_number in children:
                match = re.match(rf"^({_IDENTIFIER})[ ]*=[ ]*(.+)$", field_text)
                if not match:
                    raise NewSyntaxError(f"expected field assignment at line {field_number}")
                field, value = match.groups()
                fields.append(f"{field}: {value}")
            output.append(f"let {name}: {type_name} = {type_name} {{ {', '.join(fields)} }}")
            declared.add(name)
            index = next_index
            continue

        block_match = _BLOCK_RE.match(text)
        if block_match:
            head = block_match.group("body").strip()

            if head == "else":
                if not output or output[-1] != "}":
                    raise NewSyntaxError(f"'else' must follow a block at line {number}")
                output[-1] = "} else {"
                stack.append((depth + 1, None))
                index += 1
                continue

            typed_entity = re.match(rf"^({_IDENTIFIER})[ ]*:[ ]*({_IDENTIFIER})$", head)
            if typed_entity:
                name, type_name = typed_entity.groups()
                children, next_index = _collect_child_lines(lines, index + 1, depth)
                if not children or any(child_depth != depth + 1 for child_depth, _, _ in children):
                    raise NewSyntaxError(f"typed entity body must contain only fields at line {number}")
                fields = []
                for _, field_text, field_number in children:
                    match = re.match(rf"^({_IDENTIFIER})[ ]*=[ ]*(.+)$", field_text)
                    if not match:
                        raise NewSyntaxError(f"expected field assignment at line {field_number}")
                    field, value = match.groups()
                    fields.append(f"{field}: {value}")
                output.append(f"let {name}: {type_name} = {type_name} {{ {', '.join(fields)} }}")
                declared.add(name)
                index = next_index
                continue

            function_header = re.match(rf"^({_IDENTIFIER})[ ]*\((.*)\)(?:[ ]*->[ ]*({_IDENTIFIER}))?$", head)
            if function_header:
                name, raw_parameters, return_type = function_header.groups()
                parameters = []
                if raw_parameters.strip():
                    for raw_parameter in raw_parameters.split(","):
                        raw_parameter = raw_parameter.strip()
                        if ":" in raw_parameter:
                            parameters.append(raw_parameter)
                        else:
                            parameters.append(raw_parameter + ": Any")
                suffix = f" -> {return_type}" if return_type else " -> Any"
                output.append(f"fn {name}({', '.join(parameters)}){suffix} {{")
                stack.append((depth + 1, None))
                function_depths.append(depth + 1)
            elif head.startswith("if "):
                output.append("if " + head[3:].strip() + " {")
                stack.append((depth + 1, None))
            elif head.startswith("while "):
                output.append("while " + head[6:].strip() + " {")
                stack.append((depth + 1, None))
            elif head.startswith("when "):
                output.append("if " + head[5:].strip() + " {")
                stack.append((depth + 1, None))
            elif head.startswith("repeat ") and " in " in head:
                match = re.match(rf"repeat[ ]+({_IDENTIFIER})[ ]+in[ ]+(.+)$", head)
                if not match:
                    raise NewSyntaxError(f"invalid repeat syntax at line {number}")
                item, collection = match.groups()
                index_name = f"__axiom_index_{len(output)}"
                output.append(f"let {index_name} = 0")
                output.append(f"while {index_name} < len({collection}) {{")
                output.append(f"let {item} = {collection}[{index_name}]")
                declared.add(item)
                stack.append((depth + 1, f"@before:{index_name} = {index_name} + 1"))
            elif re.match(rf"^{_IDENTIFIER}[ ]*\([^)]*\)$", head):
                output.append("fn " + head + " {")
                stack.append((depth + 1, None))
            elif re.match(rf"^{_IDENTIFIER}[ ]*:$", text):
                output.append(f"struct {head} {{")
                stack.append((depth + 1, None))
            else:
                raise NewSyntaxError(f"unknown canonical block at line {number}: {text}")
        elif text.startswith(_DIRECTIVES):
            output.append("// canonical directive: " + text)
        else:
            translated = _translate_statement(text, declared)
            if function_depths and depth >= function_depths[-1]:
                is_statement = (
                    translated.startswith(("let ", "return ", "print(", "break", "continue"))
                    or re.match(rf"^{_IDENTIFIER}[ ]*\(", translated)
                    or re.match(rf"^{_IDENTIFIER}[ ]*=", translated)
                )
                if not is_statement:
                    translated = "return " + translated
            output.append(translated)
        index += 1

    close_to(0)
    return " ".join(output)


def parse_new(source: str) -> Program:
    from .lexer import lex
    from .parser import Parser

    raw = source.splitlines()
    normalized = []
    for number, line in enumerate(raw, 1):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        depth, text = _indent(line, number)
        normalized.append((depth, text, number))

    definitions: list[str] = []
    main_lines: list[str] = []
    index = 0
    while index < len(normalized):
        depth, text, number = normalized[index]
        if depth == 0 and _BLOCK_RE.match(text):
            head = _BLOCK_RE.match(text).group("body").strip()
            if re.match(rf"^{_IDENTIFIER}[ ]*\(.*\)(?:[ ]*->[ ]*{_IDENTIFIER})?$", head) or (head not in {"else", "if", "while", "when", "repeat"} and re.match(rf"^{_IDENTIFIER}[ ]*:$", text)):
                end = index + 1
                while end < len(normalized) and normalized[end][0] > 0:
                    end += 1
                chunk = "\n".join(item[1] if item[0] == 0 else (" " * item[0] * 4 + item[1]) for item in normalized[index:end])
                definitions.append(_canonical_to_legacy(chunk))
                index = end
                continue
        main_lines.append(" " * depth * 4 + text)
        index += 1

    pieces = definitions
    if main_lines:
        pieces.append("fn main() { " + _canonical_to_legacy("\n".join(main_lines)) + " }")
    if not pieces:
        raise NewSyntaxError("canonical AXIOM source contains no definitions or statements")
    program = Parser(lex(" ".join(pieces))).parse()
    return Program(
        program.functions,
        program.structs,
        program.module_name,
        program.imports,
        canonical=True,
    )


def looks_like_new_syntax(source: str) -> bool:
    lines = [line.strip() for line in source.splitlines() if line.strip() and not line.lstrip().startswith("#")]
    if not lines:
        return False
    legacy_markers = ("fn ", "module ", "import ", "struct ")
    return not any(line.startswith(legacy_markers) for line in lines)
