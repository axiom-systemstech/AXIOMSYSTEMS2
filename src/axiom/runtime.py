"""Runtime for executing the initial AXIOM intermediate representation."""

from __future__ import annotations

from collections.abc import Callable

from .ast import Assign, Binary, BooleanLiteral, Call, Function, If, IntegerLiteral, Let, Program, Return, StringLiteral, Unary, Variable, While
from .ir import IRProgram, LetInstruction


def execute(program: IRProgram, emit: Callable[[str], None] = print) -> None:
    """Execute an IR program using the host output function."""
    variables = {}
    for instruction in program.instructions:
        if isinstance(instruction, LetInstruction):
            variables[instruction.name] = _evaluate(instruction.value, variables)
        else:
            value = _evaluate(instruction.value, variables)
            emit(str(value).lower() if isinstance(value, bool) else str(value))


def execute_program(program: Program, emit: Callable[[str], None] = print) -> None:
    functions = {function.name: function for function in program.functions}
    _invoke(functions["main"], [], functions, emit)


def _invoke(function: Function, arguments, functions, emit):
    variables = {parameter.name: value for parameter, value in zip(function.parameters, arguments)}
    for statement in function.body:
        if isinstance(statement, Let):
            variables[statement.name] = _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, Assign):
            variables[statement.name] = _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, Return):
            return _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, If):
            branch = statement.then_body if _evaluate(statement.condition, variables, functions, emit) else statement.else_body
            result = _invoke_block(branch, variables, functions, emit)
            if result[0]:
                return result[1]
        elif isinstance(statement, While):
            while _evaluate(statement.condition, variables, functions, emit):
                returned, value = _invoke_block(statement.body, variables, functions, emit)
                if returned:
                    return value
        else:
            value = _evaluate(statement.arguments[0], variables, functions, emit)
            if statement.name == "print":
                emit(str(value).lower() if isinstance(value, bool) else str(value))
            else:
                _invoke(functions[statement.name], [_evaluate(argument, variables, functions, emit) for argument in statement.arguments], functions, emit)
    return None


def _invoke_block(statements, variables, functions, emit):
    for statement in statements:
        if isinstance(statement, Let):
            variables[statement.name] = _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, Assign):
            variables[statement.name] = _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, Return):
            return True, _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, If):
            branch = statement.then_body if _evaluate(statement.condition, variables, functions, emit) else statement.else_body
            returned, value = _invoke_block(branch, variables, functions, emit)
            if returned:
                return True, value
        elif isinstance(statement, While):
            while _evaluate(statement.condition, variables, functions, emit):
                returned, value = _invoke_block(statement.body, variables, functions, emit)
                if returned:
                    return True, value
        else:
            value = _evaluate(statement.arguments[0], variables, functions, emit)
            if statement.name == "print":
                emit(str(value).lower() if isinstance(value, bool) else str(value))
            else:
                _invoke(functions[statement.name], [_evaluate(argument, variables, functions, emit) for argument in statement.arguments], functions, emit)
    return False, None


def _evaluate(expression, variables, functions=None, emit=print):
    if isinstance(expression, (str, int, bool)):
        return expression
    if isinstance(expression, (StringLiteral, IntegerLiteral, BooleanLiteral)):
        return expression.value
    if isinstance(expression, Variable):
        return variables[expression.name]
    if isinstance(expression, Binary) and expression.operator == "+":
        return _evaluate(expression.left, variables, functions, emit) + _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == ">":
        return _evaluate(expression.left, variables, functions, emit) > _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "==":
        return _evaluate(expression.left, variables, functions, emit) == _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "-":
        return _evaluate(expression.left, variables, functions, emit) - _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "*":
        return _evaluate(expression.left, variables, functions, emit) * _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "/":
        return _evaluate(expression.left, variables, functions, emit) // _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "&&":
        return _evaluate(expression.left, variables, functions, emit) and _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "||":
        return _evaluate(expression.left, variables, functions, emit) or _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Unary) and expression.operator == "!":
        return not _evaluate(expression.operand, variables, functions, emit)
    if isinstance(expression, Unary) and expression.operator == "-":
        return -_evaluate(expression.operand, variables, functions, emit)
    if isinstance(expression, Call):
        arguments = [_evaluate(argument, variables, functions, emit) for argument in expression.arguments]
        return _invoke(functions[expression.name], arguments, functions, emit)
    raise RuntimeError("unsupported IR expression")