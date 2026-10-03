"""Canonical type model shared by AXIOM compiler and tooling."""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum

class PrimitiveType(str, Enum):
    INT = "Int"
    FLOAT = "Float"
    BOOL = "Bool"
    STRING = "String"

@dataclass(frozen=True)
class Type:
    """Stable structural representation of an AXIOM type."""

    name: str
    element: "Type | None" = None

    @classmethod
    def primitive(cls, value: PrimitiveType | str) -> "Type":
        return cls(value.value if isinstance(value, PrimitiveType) else value)

    @classmethod
    def array(cls, element: "Type") -> "Type":
        return cls("Array", element)

    @classmethod
    def named(cls, name: str) -> "Type":
        return cls(name)

    def render(self) -> str:
        if self.name == "Array" and self.element is not None:
            return f"{self.element.render()}[]"
        return self.name

    def __str__(self) -> str:
        return self.render()


def parse_type_name(name: str) -> Type:
    if not name:
        raise ValueError("type name cannot be empty")
    if name.endswith("[]"):
        return Type.array(parse_type_name(name[:-2]))
    if name in {item.value for item in PrimitiveType}:
        return Type.primitive(name)
    return Type.named(name)


def type_name(value: Type) -> str:
    return value.render()
