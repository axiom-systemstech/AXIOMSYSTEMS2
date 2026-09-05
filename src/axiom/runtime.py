"""Runtime for executing the initial AXIOM intermediate representation."""

from __future__ import annotations

from collections.abc import Callable

from .ast import ArrayLiteral, Assign, Binary, BooleanLiteral, Break, Call, Continue, For, Function, If, Index, IntegerLiteral, Let, Program, Return, StringLiteral, Unary, Variable, While
from .ir import BreakInstruction, CallInstruction, ContinueInstruction, ForInstruction, IRFunction, IRProgram, IfInstruction, LetInstruction, ReturnInstruction, SetInstruction, WhileInstruction


def execute(program: IRProgram, emit: Callable[[str], None] = print) -> None:
    """Execute an IR program using the host output function."""
    functions = {function.name: function for function in program.functions}
    main = IRFunction("main", [], program.instructions)
    _invoke_ir(main, [], functions | {"main": main}, emit)


def _invoke_ir(function, arguments, functions, emit):
    variables = {name: value for name, value in zip(function.parameters, arguments)}
    returned, value = _execute_block(function.body, variables, functions, emit)
    return value if returned else None


def _execute_block(instructions, variables, functions, emit):
    for instruction in instructions:
        if isinstance(instruction, LetInstruction):
            variables[instruction.name] = _evaluate(instruction.value, variables, functions, emit)
        elif isinstance(instruction, SetInstruction):
            _assign(instruction.target, _evaluate(instruction.value, variables, functions, emit), variables, functions, emit)
        elif isinstance(instruction, IfInstruction):
            branch = instruction.then_body if _evaluate(instruction.condition, variables, functions, emit) else instruction.else_body
            status, value = _execute_block(branch, variables, functions, emit)
            if status is _BREAK or status is _CONTINUE:
                return status, None
            if status:
                return True, value
        elif isinstance(instruction, WhileInstruction):
            while _evaluate(instruction.condition, variables, functions, emit):
                status, value = _execute_block(instruction.body, variables, functions, emit)
                if status is _BREAK: break
                if status is _CONTINUE: continue
                if status:
                    return True, value
        elif isinstance(instruction, ForInstruction):
            _execute_block(instruction.initializer, variables, functions, emit)
            while _evaluate(instruction.condition, variables, functions, emit):
                status, value = _execute_block(instruction.body, variables, functions, emit)
                if status is _BREAK: break
                if status is _CONTINUE:
                    _execute_block(instruction.update, variables, functions, emit)
                    continue
                if status:
                    return True, value
                _execute_block(instruction.update, variables, functions, emit)
        elif isinstance(instruction, BreakInstruction):
            return _BREAK, None
        elif isinstance(instruction, ContinueInstruction):
            return _CONTINUE, None
        elif isinstance(instruction, CallInstruction):
            arguments = [_evaluate(argument, variables, functions, emit) for argument in instruction.arguments]
            _invoke_ir(functions[instruction.name], arguments, functions, emit)
        elif isinstance(instruction, ReturnInstruction):
            return True, _evaluate(instruction.value, variables, functions, emit)
        else:
            value = _evaluate(instruction.value, variables, functions, emit)
            emit(str(value).lower() if isinstance(value, bool) else str(value))
    return False, None

_BREAK = object()
_CONTINUE = object()


def execute_program(program: Program, emit: Callable[[str], None] = print) -> None:
    functions = {function.name: function for function in program.functions}
    _invoke(functions["main"], [], functions, emit)


def _invoke(function: Function, arguments, functions, emit):
    variables = {parameter.name: value for parameter, value in zip(function.parameters, arguments)}
    for statement in function.body:
        if isinstance(statement, Let):
            variables[statement.name] = _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, Assign):
            _assign(statement.target, _evaluate(statement.value, variables, functions, emit), variables, functions, emit)
        elif isinstance(statement, Return):
            return _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, If):
            branch = statement.then_body if _evaluate(statement.condition, variables, functions, emit) else statement.else_body
            status, value = _invoke_block(branch, variables, functions, emit)
            if status is _BREAK or status is _CONTINUE:
                return status
            if status:
                return value
        elif isinstance(statement, While):
            while _evaluate(statement.condition, variables, functions, emit):
                status, value = _invoke_block(statement.body, variables, functions, emit)
                if status is _BREAK: break
                if status is _CONTINUE: continue
                if status:
                    return value
        elif isinstance(statement, For):
            if statement.initializer is not None:
                if isinstance(statement.initializer, Let):
                    variables[statement.initializer.name] = _evaluate(statement.initializer.value, variables, functions, emit)
                elif isinstance(statement.initializer, Assign):
                    _assign(statement.initializer.target, _evaluate(statement.initializer.value, variables, functions, emit), variables, functions, emit)
            while _evaluate(statement.condition, variables, functions, emit):
                status, value = _invoke_block(statement.body, variables, functions, emit)
                if status is _BREAK: break
                if status is _CONTINUE:
                    if statement.update is not None:
                        _assign(statement.update.target, _evaluate(statement.update.value, variables, functions, emit), variables, functions, emit)
                    continue
                if status:
                    return value
                if statement.update is not None:
                    _assign(statement.update.target, _evaluate(statement.update.value, variables, functions, emit), variables, functions, emit)
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
            _assign(statement.target, _evaluate(statement.value, variables, functions, emit), variables, functions, emit)
        elif isinstance(statement, Return):
            return True, _evaluate(statement.value, variables, functions, emit)
        elif isinstance(statement, If):
            branch = statement.then_body if _evaluate(statement.condition, variables, functions, emit) else statement.else_body
            status, value = _invoke_block(branch, variables, functions, emit)
            if status is _BREAK or status is _CONTINUE:
                return status, None
            if status:
                return True, value
        elif isinstance(statement, While):
            while _evaluate(statement.condition, variables, functions, emit):
                status, value = _invoke_block(statement.body, variables, functions, emit)
                if status is _BREAK: break
                if status is _CONTINUE: continue
                if status:
                    return True, value
        elif isinstance(statement, For):
            if statement.initializer is not None:
                if isinstance(statement.initializer, Let):
                    variables[statement.initializer.name] = _evaluate(statement.initializer.value, variables, functions, emit)
                elif isinstance(statement.initializer, Assign):
                    _assign(statement.initializer.target, _evaluate(statement.initializer.value, variables, functions, emit), variables, functions, emit)
            while _evaluate(statement.condition, variables, functions, emit):
                status, value = _invoke_block(statement.body, variables, functions, emit)
                if status is _BREAK: return False, None
                if status is _CONTINUE:
                    if statement.update is not None:
                        _assign(statement.update.target, _evaluate(statement.update.value, variables, functions, emit), variables, functions, emit)
                    continue
                if status:
                    return True, value
                if statement.update is not None:
                    _assign(statement.update.target, _evaluate(statement.update.value, variables, functions, emit), variables, functions, emit)
        elif isinstance(statement, Break):
            return _BREAK, None
        elif isinstance(statement, Continue):
            return _CONTINUE, None
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
    if isinstance(expression, ArrayLiteral):
        return [_evaluate(element, variables, functions, emit) for element in expression.elements]
    if isinstance(expression, Variable):
        return variables[expression.name]
    if isinstance(expression, Index):
        target = _evaluate(expression.target, variables, functions, emit)
        index = _evaluate(expression.index, variables, functions, emit)
        try:
            if index < 0:
                raise IndexError
            return target[index]
        except (IndexError, TypeError):
            raise RuntimeError("index out of bounds") from None
    if isinstance(expression, Binary) and expression.operator == "+":
        left = _evaluate(expression.left, variables, functions, emit)
        right = _evaluate(expression.right, variables, functions, emit)
        return left + right
    if isinstance(expression, Binary) and expression.operator == ">":
        return _evaluate(expression.left, variables, functions, emit) > _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == ">=":
        return _evaluate(expression.left, variables, functions, emit) >= _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "<":
        return _evaluate(expression.left, variables, functions, emit) < _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "<=":
        return _evaluate(expression.left, variables, functions, emit) <= _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "==":
        return _evaluate(expression.left, variables, functions, emit) == _evaluate(expression.right, variables, functions, emit)
    if isinstance(expression, Binary) and expression.operator == "!=":
        return _evaluate(expression.left, variables, functions, emit) != _evaluate(expression.right, variables, functions, emit)
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
        if isinstance(functions[expression.name], IRFunction):
            return _invoke_ir(functions[expression.name], arguments, functions, emit)
        return _invoke(functions[expression.name], arguments, functions, emit)
    raise RuntimeError("unsupported IR expression")


def _assign(target, value, variables, functions=None, emit=print):
    if isinstance(target, Variable):
        variables[target.name] = value
        return
    if isinstance(target, Index):
        container = _evaluate(target.target, variables, functions, emit)
        index = _evaluate(target.index, variables, functions, emit)
        try:
            if index < 0:
                raise IndexError
            container[index] = value
        except (IndexError, TypeError):
            raise RuntimeError("index out of bounds") from None
        return
    raise RuntimeError("invalid assignment target")