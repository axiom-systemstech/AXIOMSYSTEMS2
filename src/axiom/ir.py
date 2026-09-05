"""Portable intermediate representation for the initial AXIOM compiler."""

from __future__ import annotations

from dataclasses import dataclass

from .ast import ArrayLiteral, Assign, Binary, BooleanLiteral, Expression, Index, IntegerLiteral, Let, Program, StringLiteral, Unary, Variable

IRValue = Expression | str | int | bool


@dataclass(frozen=True)
class PrintInstruction:
    value: IRValue


@dataclass(frozen=True)
class LetInstruction:
    name: str
    value: IRValue


@dataclass(frozen=True)
class SetInstruction:
    target: Expression
    value: IRValue


Instruction = PrintInstruction | LetInstruction | SetInstruction


@dataclass(frozen=True)
class IRProgram:
    instructions: list[Instruction]

    def render(self) -> str:
        lines = ["AXIOM-IR 0.1"]
        for instruction in self.instructions:
            if isinstance(instruction, LetInstruction):
                lines.append(f"LET {instruction.name} = {_render_expression(instruction.value)}")
            elif isinstance(instruction, SetInstruction):
                lines.append(
                    f"SET {_render_expression(instruction.target)} = "
                    f"{_render_expression(instruction.value)}"
                )
            else:
                lines.append(f"PRINT {_render_expression(instruction.value)}")
        return "\n".join(lines) + "\n"


def lower(program: Program) -> IRProgram:
    instructions = []
    for function in program.functions:
        if function.name == "main":
            for statement in function.body:
                if isinstance(statement, Let):
                    instructions.append(LetInstruction(statement.name, statement.value))
                elif isinstance(statement, Assign):
                    instructions.append(SetInstruction(statement.target, statement.value))
                else:
                    instructions.append(PrintInstruction(statement.arguments[0]))
    return IRProgram(instructions)


def _render_expression(expression: IRValue) -> str:
    if isinstance(expression, StringLiteral):
        return repr(expression.value)
    if isinstance(expression, IntegerLiteral):
        return str(expression.value)
    if isinstance(expression, BooleanLiteral):
        return str(expression.value).lower()
    if isinstance(expression, ArrayLiteral):
        elements = ", ".join(_render_expression(element) for element in expression.elements)
        return f"[{elements}]"
    if isinstance(expression, Variable):
        return expression.name
    if isinstance(expression, Index):
        return f"{_render_expression(expression.target)}[{_render_expression(expression.index)}]"
    if isinstance(expression, Unary):
        return f"{expression.operator}{_render_expression(expression.operand)}"
    if hasattr(expression, "name") and hasattr(expression, "arguments"):
        arguments = ", ".join(_render_expression(argument) for argument in expression.arguments)
        return f"{expression.name}({arguments})"
    if isinstance(expression, Binary):
        return f"{_render_expression(expression.left)} {expression.operator} {_render_expression(expression.right)}"
    return repr(expression)