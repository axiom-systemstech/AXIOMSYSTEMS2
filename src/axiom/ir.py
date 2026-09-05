"""Portable intermediate representation for the initial AXIOM compiler."""

from __future__ import annotations

from dataclasses import dataclass, field

from .ast import ArrayLiteral, Assign, Binary, BooleanLiteral, Break, Call, Continue, Expression, For, If, Index, IntegerLiteral, Let, Program, Return, StringLiteral, Unary, Variable, While

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


@dataclass(frozen=True)
class IfInstruction:
    condition: IRValue
    then_body: list["Instruction"]
    else_body: list["Instruction"]


@dataclass(frozen=True)
class WhileInstruction:
    condition: IRValue
    body: list["Instruction"]


@dataclass(frozen=True)
class ForInstruction:
    initializer: list["Instruction"]
    condition: IRValue
    update: list["Instruction"]
    body: list["Instruction"]


@dataclass(frozen=True)
class CallInstruction:
    name: str
    arguments: list[Expression]


@dataclass(frozen=True)
class ReturnInstruction:
    value: IRValue


@dataclass(frozen=True)
class BreakInstruction:
    pass


@dataclass(frozen=True)
class ContinueInstruction:
    pass


Instruction = PrintInstruction | LetInstruction | SetInstruction | IfInstruction | WhileInstruction | ForInstruction | CallInstruction | ReturnInstruction | BreakInstruction | ContinueInstruction


@dataclass(frozen=True)
class IRFunction:
    name: str
    parameters: list[str]
    body: list[Instruction]


@dataclass(frozen=True)
class IRProgram:
    instructions: list[Instruction]
    functions: list[IRFunction] = field(default_factory=list)

    def render(self) -> str:
        lines = ["AXIOM-IR 0.1"]
        if self.functions:
            lines.append("FUNCTION main()")
            _render_block(self.instructions, lines, 1)
            lines.append("END FUNCTION")
            for function in self.functions:
                parameters = ", ".join(function.parameters)
                lines.append(f"FUNCTION {function.name}({parameters})")
                _render_block(function.body, lines, 1)
                lines.append("END FUNCTION")
        else:
            _render_block(self.instructions, lines, 0)
        return "\n".join(lines) + "\n"


def lower(program: Program) -> IRProgram:
    instructions = []
    functions = []
    for function in program.functions:
        if function.name == "main":
            instructions.extend(_lower_block(function.body))
        else:
            functions.append(IRFunction(function.name, [parameter.name for parameter in function.parameters], _lower_block(function.body)))
    return IRProgram(instructions, functions)


def _lower_block(statements) -> list[Instruction]:
    instructions = []
    for statement in statements:
        if isinstance(statement, Let):
            instructions.append(LetInstruction(statement.name, statement.value))
        elif isinstance(statement, Assign):
            instructions.append(SetInstruction(statement.target, statement.value))
        elif isinstance(statement, If):
            instructions.append(
                IfInstruction(
                    statement.condition,
                    _lower_block(statement.then_body),
                    _lower_block(statement.else_body),
                )
            )
        elif isinstance(statement, While):
            instructions.append(WhileInstruction(statement.condition, _lower_block(statement.body)))
        elif isinstance(statement, For):
            initializer = []
            if statement.initializer is not None:
                initializer.extend(_lower_block([statement.initializer]))
            body = _lower_block(statement.body)
            update = []
            if statement.update is not None:
                update = [SetInstruction(statement.update.target, statement.update.value)]
            instructions.append(ForInstruction(initializer, statement.condition, update, body))
        elif isinstance(statement, Break):
            instructions.append(BreakInstruction())
        elif isinstance(statement, Continue):
            instructions.append(ContinueInstruction())
        elif isinstance(statement, Return):
            instructions.append(ReturnInstruction(statement.value))
        elif isinstance(statement, Call):
            if statement.name == "print":
                instructions.append(PrintInstruction(statement.arguments[0]))
            else:
                instructions.append(CallInstruction(statement.name, statement.arguments))
        else:
            raise ValueError(f"unsupported statement in IR lowering: {type(statement).__name__}")
    return instructions


def _render_block(instructions: list[Instruction], lines: list[str], depth: int) -> None:
    prefix = "  " * depth
    for instruction in instructions:
        if isinstance(instruction, LetInstruction):
            lines.append(f"{prefix}LET {instruction.name} = {_render_expression(instruction.value)}")
        elif isinstance(instruction, SetInstruction):
            lines.append(
                f"{prefix}SET {_render_expression(instruction.target)} = "
                f"{_render_expression(instruction.value)}"
            )
        elif isinstance(instruction, IfInstruction):
            lines.append(f"{prefix}IF {_render_expression(instruction.condition)}")
            _render_block(instruction.then_body, lines, depth + 1)
            if instruction.else_body:
                lines.append(f"{prefix}ELSE")
                _render_block(instruction.else_body, lines, depth + 1)
            lines.append(f"{prefix}END")
        elif isinstance(instruction, WhileInstruction):
            lines.append(f"{prefix}WHILE {_render_expression(instruction.condition)}")
            _render_block(instruction.body, lines, depth + 1)
            lines.append(f"{prefix}END")
        elif isinstance(instruction, ForInstruction):
            if instruction.initializer:
                _render_block(instruction.initializer, lines, depth)
            lines.append(f"{prefix}FOR {_render_expression(instruction.condition)}")
            _render_block(instruction.body, lines, depth + 1)
            if instruction.update:
                lines.append(f"{prefix}UPDATE")
                _render_block(instruction.update, lines, depth + 1)
            lines.append(f"{prefix}END FOR")
        elif isinstance(instruction, CallInstruction):
            arguments = ", ".join(_render_expression(argument) for argument in instruction.arguments)
            lines.append(f"{prefix}CALL {instruction.name}({arguments})")
        elif isinstance(instruction, ReturnInstruction):
            lines.append(f"{prefix}RETURN {_render_expression(instruction.value)}")
        elif isinstance(instruction, BreakInstruction):
            lines.append(f"{prefix}BREAK")
        elif isinstance(instruction, ContinueInstruction):
            lines.append(f"{prefix}CONTINUE")
        else:
            lines.append(f"{prefix}PRINT {_render_expression(instruction.value)}")


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