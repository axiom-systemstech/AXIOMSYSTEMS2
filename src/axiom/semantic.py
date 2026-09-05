"""Semantic checks for the first AXIOM language slice."""

from __future__ import annotations

from .ast import ArrayLiteral, Assign, Binary, BooleanLiteral, Call, Function, If, Index, IntegerLiteral, Let, Program, Return, StringLiteral, Unary, Variable, While


class SemanticError(ValueError):
    """Raised when a syntactically valid program is not meaningful."""


_BUILTIN_TYPES = {"Int", "Bool", "String"}


def _is_known_type(type_name: str) -> bool:
    return type_name.rstrip("[]") in _BUILTIN_TYPES


def _types_compatible(expected: str, actual: str) -> bool:
    return actual == expected or (actual == "Array" and expected.endswith("[]"))


def analyze(program: Program) -> None:
    function_names: set[str] = set()
    signatures = {}
    for function in program.functions:
        if function.name in function_names:
            raise SemanticError(f"duplicate function '{function.name}'")
        function_names.add(function.name)
        parameter_types = [parameter.type_name for parameter in function.parameters]
        if any(not _is_known_type(type_name) for type_name in parameter_types):
            raise SemanticError(f"function '{function.name}' uses an unknown parameter type")
        if function.return_type is not None and not _is_known_type(function.return_type):
            raise SemanticError(f"function '{function.name}' uses an unknown return type")
        if len({parameter.name for parameter in function.parameters}) != len(function.parameters):
            raise SemanticError(f"function '{function.name}' has duplicate parameters")
        signatures[function.name] = (parameter_types, function.return_type)

    main = next((function for function in program.functions if function.name == "main"), None)
    if main is None:
        raise SemanticError("program must define 'main'")

    for function in program.functions:
        _check_function(function, signatures)


def _check_function(function: Function, signatures) -> None:
    variables = {parameter.name: parameter.type_name for parameter in function.parameters}
    returned = False
    for statement in function.body:
        if isinstance(statement, If):
            condition_type = _expression_type(statement.condition, variables, signatures)
            if condition_type != "Bool":
                raise SemanticError("if condition must be Bool")
            _check_block(statement.then_body, variables.copy(), signatures, function)
            _check_block(statement.else_body, variables.copy(), signatures, function)
            continue
        if isinstance(statement, While):
            if _expression_type(statement.condition, variables, signatures) != "Bool":
                raise SemanticError("while condition must be Bool")
            _check_block(statement.body, variables.copy(), signatures, function)
            continue
        if isinstance(statement, Assign):
            target_type = _expression_type(statement.target, variables, signatures)
            value_type = _expression_type(statement.value, variables, signatures)
            if value_type != target_type:
                raise SemanticError(f"assignment expects {target_type}, got {value_type}")
            continue
        if isinstance(statement, Let):
            value_type = _expression_type(statement.value, variables, signatures)
            if statement.type_name is not None and not _types_compatible(statement.type_name, value_type):
                raise SemanticError(
                    f"variable '{statement.name}' expects {statement.type_name}, got {value_type}"
                )
            if statement.name in variables:
                raise SemanticError(f"variable '{statement.name}' already declared")
            variables[statement.name] = statement.type_name or value_type
            continue
        if not isinstance(statement, Call):
            if isinstance(statement, Return):
                if function.return_type is None:
                    raise SemanticError(f"function '{function.name}' cannot return a value")
                value_type = _expression_type(statement.value, variables, signatures)
                if not _types_compatible(function.return_type, value_type):
                    raise SemanticError(f"function '{function.name}' returns {value_type}, expected {function.return_type}")
                returned = True
                continue
            raise SemanticError(f"unsupported statement in '{function.name}'")
        if statement.name == "print":
            if len(statement.arguments) != 1:
                raise SemanticError("print expects exactly one argument")
            _expression_type(statement.arguments[0], variables, signatures)
            continue
        _check_call(statement, variables, signatures)
    if function.return_type is not None and not returned:
        raise SemanticError(f"function '{function.name}' must return {function.return_type}")


def _check_block(statements, variables, signatures, function):
    for statement in statements:
        if isinstance(statement, Let):
            value_type = _expression_type(statement.value, variables, signatures)
            if statement.type_name is not None and not _types_compatible(statement.type_name, value_type):
                raise SemanticError(f"variable '{statement.name}' expects {statement.type_name}, got {value_type}")
            variables[statement.name] = statement.type_name or value_type
        elif isinstance(statement, Return):
            if function.return_type is None:
                raise SemanticError(f"function '{function.name}' cannot return a value")
            if _expression_type(statement.value, variables, signatures) != function.return_type:
                raise SemanticError(f"function '{function.name}' returns an incompatible value")
        elif isinstance(statement, If):
            if _expression_type(statement.condition, variables, signatures) != "Bool":
                raise SemanticError("if condition must be Bool")
            _check_block(statement.then_body, variables.copy(), signatures, function)
            _check_block(statement.else_body, variables.copy(), signatures, function)
        elif isinstance(statement, While):
            if _expression_type(statement.condition, variables, signatures) != "Bool":
                raise SemanticError("while condition must be Bool")
            _check_block(statement.body, variables.copy(), signatures, function)
        elif isinstance(statement, Assign):
            target_type = _expression_type(statement.target, variables, signatures)
            if _expression_type(statement.value, variables, signatures) != target_type:
                raise SemanticError("assignment has incompatible type")
        elif isinstance(statement, Call):
            if statement.name == "print":
                if len(statement.arguments) != 1:
                    raise SemanticError("print expects exactly one argument")
                _expression_type(statement.arguments[0], variables, signatures)
            else:
                _check_call(statement, variables, signatures)


def _check_call(call: Call, variables: dict[str, str], signatures) -> str:
    if call.name not in signatures:
        raise SemanticError(f"unknown function '{call.name}'")
    parameter_types, return_type = signatures[call.name]
    if len(call.arguments) != len(parameter_types):
        raise SemanticError(f"function '{call.name}' expects {len(parameter_types)} arguments")
    argument_types = [_expression_type(argument, variables, signatures) for argument in call.arguments]
    if argument_types != parameter_types:
        raise SemanticError(f"function '{call.name}' received {argument_types}, expected {parameter_types}")
    if return_type is None:
        raise SemanticError(f"function '{call.name}' has no return value")
    return return_type


def _expression_type(expression, variables: dict[str, str], signatures) -> str:
    if isinstance(expression, StringLiteral):
        return "String"
    if isinstance(expression, IntegerLiteral):
        return "Int"
    if isinstance(expression, BooleanLiteral):
        return "Bool"
    if isinstance(expression, ArrayLiteral):
        if not expression.elements:
            return "Array"
        element_types = [_expression_type(element, variables, signatures) for element in expression.elements]
        if any(element_type != element_types[0] for element_type in element_types[1:]):
            raise SemanticError("array elements must share the same type")
        return f"{element_types[0]}[]"
    if isinstance(expression, Variable):
        if expression.name not in variables:
            raise SemanticError(f"unknown variable '{expression.name}'")
        return variables[expression.name]
    if isinstance(expression, Index):
        target_type = _expression_type(expression.target, variables, signatures)
        if _expression_type(expression.index, variables, signatures) != "Int":
            raise SemanticError("array index requires Int")
        if not target_type.endswith("[]"):
            raise SemanticError("index requires an array")
        return target_type[:-2]
    if isinstance(expression, Call):
        return _check_call(expression, variables, signatures)
    if isinstance(expression, Binary):
        left_type = _expression_type(expression.left, variables, signatures)
        right_type = _expression_type(expression.right, variables, signatures)
        if expression.operator == "+" and left_type == right_type == "Int":
            return "Int"
        if expression.operator in {">", ">=", "<", "<="} and left_type == right_type == "Int":
            return "Bool"
        if expression.operator in {"==", "!="} and left_type == right_type:
            return "Bool"
        if expression.operator in {"-", "*", "/"} and left_type == right_type == "Int":
            return "Int"
        if expression.operator in {"&&", "||"} and left_type == right_type == "Bool":
            return "Bool"
        raise SemanticError(f"operator {expression.operator!r} does not support {left_type} and {right_type}")
    if isinstance(expression, Unary):
        operand_type = _expression_type(expression.operand, variables, signatures)
        if expression.operator == "!" and operand_type == "Bool":
            return "Bool"
        if expression.operator == "-" and operand_type == "Int":
            return "Int"
        raise SemanticError(f"operator {expression.operator!r} does not support {operand_type}")
    raise SemanticError("unsupported expression")