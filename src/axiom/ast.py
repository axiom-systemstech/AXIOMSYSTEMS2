"""Abstract syntax tree nodes for the first AXIOM syntax slice."""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass(frozen=True)
class Program:
    functions: list["Function"]


@dataclass(frozen=True)
class Function:
    name: str
    body: list["Statement"]
    parameters: list["Parameter"] = field(default_factory=list)
    return_type: str | None = None


@dataclass(frozen=True)
class Parameter:
    name: str
    type_name: str


@dataclass(frozen=True)
class Call:
    name: str
    arguments: list["Expression"]


@dataclass(frozen=True)
class StringLiteral:
    value: str


@dataclass(frozen=True)
class IntegerLiteral:
    value: int


@dataclass(frozen=True)
class BooleanLiteral:
    value: bool


@dataclass(frozen=True)
class ArrayLiteral:
    elements: list["Expression"]


@dataclass(frozen=True)
class Variable:
    name: str


@dataclass(frozen=True)
class Binary:
    left: "Expression"
    operator: str
    right: "Expression"


@dataclass(frozen=True)
class Unary:
    operator: str
    operand: "Expression"


@dataclass(frozen=True)
class Let:
    name: str
    type_name: str | None
    value: "Expression"


@dataclass(frozen=True)
class Return:
    value: "Expression"


@dataclass(frozen=True)
class If:
    condition: "Expression"
    then_body: list["Statement"]
    else_body: list["Statement"] = field(default_factory=list)


@dataclass(frozen=True)
class Assign:
    target: "Expression"
    value: "Expression"


@dataclass(frozen=True)
class While:
    condition: "Expression"
    body: list["Statement"]


@dataclass(frozen=True)
class For:
    initializer: "Statement | None"
    condition: "Expression"
    update: "Assign | None"
    body: list["Statement"]


Statement = Call | Let | Return | If | Assign | While | For


@dataclass(frozen=True)
class Index:
    target: "Expression"
    index: "Expression"


Expression = StringLiteral | IntegerLiteral | BooleanLiteral | ArrayLiteral | Variable | Binary | Unary | Call | Index