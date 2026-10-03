"""Language-neutral semantic facts produced by the AXIOM 0.1 compiler.

This layer deliberately sits between syntax and the historical IR.  It records
meaning without choosing a machine representation.
"""

from __future__ import annotations

from dataclasses import dataclass, field
from enum import Enum

from .ast import Assign, Binary, Call, FieldAccess, For, Function, If, Index, Let, Program, Return, StructLiteral, Unary, Variable, While


class EntityKind(str, Enum):
    VALUE = "VALUE"
    DATA = "DATA"
    RESOURCE = "RESOURCE"
    CAPABILITY = "CAPABILITY"
    KNOWLEDGE = "KNOWLEDGE"


class Relation(str, Enum):
    PRODUCE = "PRODUCE"
    CONSUME = "CONSUME"
    TRANSFORM = "TRANSFORM"
    READ = "READ"
    WRITE = "WRITE"
    DEPEND = "DEPEND"
    CALL = "CALL"
    COMMUNICATE = "COMMUNICATE"
    CONTROL = "CONTROL"
    CREATE = "CREATE"
    RELEASE = "RELEASE"
    CONTAIN = "CONTAIN"
    MEASURE = "MEASURE"


@dataclass(frozen=True)
class EntityFact:
    name: str
    kind: EntityKind
    type_name: str | None = None
    scope: str = "main"


@dataclass(frozen=True)
class RelationFact:
    source: str
    relation: Relation
    target: str
    scope: str = "main"


@dataclass(frozen=True)
class EffectFact:
    name: str
    scope: str
    source: str
    transitive: bool = False


@dataclass(frozen=True)
class ResourceFact:
    name: str
    type_name: str | None
    created_in: str
    consumers: tuple[str, ...] = ()
    released: bool = False


@dataclass(frozen=True)
class CapabilityFact:
    name: str
    source: str
    scope: str
    allowed: bool = True


@dataclass(frozen=True)
class ResourceFlowFact:
    resource: str
    scope: str
    state: str
    consumers: tuple[str, ...] = ()
    aliases: tuple[str, ...] = ()
    transferred_to: str | None = None
    released_by: str | None = None


@dataclass(frozen=True)
class SemanticModel:
    entities: tuple[EntityFact, ...] = ()
    relations: tuple[RelationFact, ...] = ()
    resources: tuple[ResourceFact, ...] = ()
    resource_flow: tuple[ResourceFlowFact, ...] = ()
    effects: tuple[EffectFact, ...] = ()
    capabilities: tuple[str, ...] = ()
    requirements: tuple[str, ...] = ()
    allowed_capabilities: tuple[str, ...] = ()
    preferences: tuple[str, ...] = ()
    restrictions: tuple[str, ...] = ()
    modes: tuple[str, ...] = ()
    contracts: tuple[str, ...] = ()
    capability_facts: tuple[CapabilityFact, ...] = ()
    diagnostics: tuple[str, ...] = ()

    def effects_for(self, scope: str) -> tuple[EffectFact, ...]:
        return tuple(effect for effect in self.effects if effect.scope == scope)

    def relations_for(self, scope: str) -> tuple[RelationFact, ...]:
        return tuple(relation for relation in self.relations if relation.scope == scope)


_CALL_EFFECTS = {
    "print": ("terminal.write", "terminal.write"),
    "len": (None, None),
    "abs": (None, None),
    "min": (None, None),
    "max": (None, None),
}


def build_semantic_model(program: Program) -> SemanticModel:
    entities: list[EntityFact] = []
    relations: list[RelationFact] = []
    resources: list[ResourceFact] = []
    effects: list[EffectFact] = []
    capabilities: set[str] = set()

    for struct in program.structs:
        entities.append(EntityFact(struct.name, EntityKind.DATA, struct.name, "type"))

    for function in program.functions:
        _visit_function(function, entities, relations, resources, effects, capabilities)

    # Calls through user functions propagate their effects to callers.
    effects = _propagate_call_effects(program, effects)
    resource_flow, resource_diagnostics = _analyze_resource_flow(program, resources)

    explicit_needs = {item.value for item in program.directives if item.kind == "needs"}
    allowed = {item.value for item in program.directives if item.kind == "can"}
    preferences = tuple(item.value for item in program.directives if item.kind == "prefer")
    restrictions = tuple(item.value for item in program.directives if item.kind == "restrict")
    modes = tuple(item.value for item in program.directives if item.kind == "mode")
    contracts = tuple(item.value for item in program.directives if item.kind == "prove")
    requirements = explicit_needs | capabilities
    capability_facts = [
        CapabilityFact(name, "effect-inference", "program", not allowed or name in allowed)
        for name in sorted(capabilities)
    ]
    diagnostics = tuple(
        [
            f"capability '{fact.name}' is not allowed by the program"
            for fact in capability_facts
            if not fact.allowed
        ]
        + list(resource_diagnostics)
    )

    return SemanticModel(
        entities=tuple(entities),
        relations=tuple(relations),
        resources=tuple(resources),
        resource_flow=tuple(resource_flow),
        effects=tuple(sorted(effects, key=lambda item: (item.scope, item.name, item.source, item.transitive))),
        capabilities=tuple(sorted(capabilities)),
        requirements=tuple(sorted(requirements)),
        allowed_capabilities=tuple(sorted(allowed)),
        preferences=preferences,
        restrictions=restrictions,
        modes=modes,
        contracts=contracts,
        capability_facts=tuple(capability_facts),
        diagnostics=diagnostics,
    )


def _analyze_resource_flow(program: Program, resources: list[ResourceFact]) -> tuple[list[ResourceFlowFact], tuple[str, ...]]:
    known = {(resource.created_in, resource.name) for resource in resources}
    flow: list[ResourceFlowFact] = []
    diagnostics: list[str] = []
    for function in program.functions:
        roots = {name: name for scope, name in known if scope == function.name}
        state = {name: "ACTIVE" for name in roots.values()}
        aliases = {name: set() for name in roots.values()}
        consumers = {name: [] for name in roots.values()}
        released_by = {}
        _walk_resource_block(function.body, function.name, roots, state, aliases, consumers, released_by, diagnostics)
        for name in sorted(consumers):
            non_lifecycle = [item for item in consumers[name] if item not in {"release", "close", "free", "drop"}]
            if len(set(non_lifecycle)) > 1 and state[name] == "ACTIVE":
                state[name] = "SHARED"
            flow.append(ResourceFlowFact(
                resource=name,
                scope=function.name,
                state=state[name],
                consumers=tuple(consumers[name]),
                aliases=tuple(sorted(aliases[name])),
                released_by=released_by.get(name),
            ))
    return flow, tuple(diagnostics)


def _walk_resource_block(statements, scope, roots, state, aliases, consumers, released_by, diagnostics):
    release_calls = {"release", "close", "free", "drop"}
    for statement in statements:
        if isinstance(statement, Let) and isinstance(statement.value, (Variable, FieldAccess, Index)):
            source = _resource_argument_name(statement.value)
            if source in roots:
                root = roots[source]
                roots[statement.name] = root
                aliases[root].add(statement.name)
            continue
        if isinstance(statement, Assign) and isinstance(statement.target, Variable):
            source = _resource_argument_name(statement.value)
            if source in roots:
                root = roots[source]
                roots[statement.target.name] = root
                aliases[root].add(statement.target.name)
            continue
        if isinstance(statement, Call):
            for argument in statement.arguments:
                name = _resource_argument_name(argument)
                if name not in roots:
                    continue
                root = roots[name]
                if state[root] in {"RELEASED", "MAYBE_RELEASED"}:
                    diagnostics.append(f"resource '{root}' is used after release in '{scope}'")
                    continue
                consumers[root].append(statement.name)
                if statement.name in release_calls:
                    state[root] = "RELEASED"
                    released_by[root] = statement.name
            continue
        if isinstance(statement, If):
            before = dict(state)
            then_state = dict(state)
            else_state = dict(state)
            _walk_resource_block(statement.then_body, scope, dict(roots), then_state, aliases, consumers, released_by, diagnostics)
            _walk_resource_block(statement.else_body, scope, dict(roots), else_state, aliases, consumers, released_by, diagnostics)
            for root in state:
                left, right = then_state[root], else_state[root]
                state[root] = left if left == right else ("MAYBE_RELEASED" if "RELEASED" in {left, right} else before[root])
            continue
        if isinstance(statement, While):
            before = dict(state)
            loop_state = dict(state)
            _walk_resource_block(statement.body, scope, dict(roots), loop_state, aliases, consumers, released_by, diagnostics)
            for root in state:
                if loop_state[root] == "RELEASED" and before[root] == "ACTIVE":
                    state[root] = "MAYBE_RELEASED"
                elif loop_state[root] != before[root]:
                    state[root] = "MAYBE_RELEASED"
            continue
        if isinstance(statement, For):
            if statement.initializer is not None:
                _walk_resource_block([statement.initializer], scope, roots, state, aliases, consumers, released_by, diagnostics)
            before = dict(state)
            loop_state = dict(state)
            _walk_resource_block(statement.body, scope, dict(roots), loop_state, aliases, consumers, released_by, diagnostics)
            for root in state:
                if loop_state[root] != before[root]:
                    state[root] = "MAYBE_RELEASED"


def _visit_function(function: Function, entities, relations, resources, effects, capabilities) -> None:
    scope = function.name
    for parameter in function.parameters:
        kind = EntityKind.RESOURCE if _looks_like_resource(parameter.name, parameter.type_name) else EntityKind.VALUE
        entities.append(EntityFact(parameter.name, kind, parameter.type_name, scope))
        if kind is EntityKind.RESOURCE:
            resources.append(ResourceFact(parameter.name, parameter.type_name, scope))

    for statement in function.body:
        _visit_statement(statement, scope, entities, relations, resources, effects, capabilities)


def _visit_statement(statement, scope, entities, relations, resources, effects, capabilities) -> None:
    if isinstance(statement, Let):
        kind = EntityKind.RESOURCE if _looks_like_resource(statement.name, statement.type_name) else EntityKind.VALUE
        entities.append(EntityFact(statement.name, kind, statement.type_name, scope))
        relations.append(RelationFact(statement.name, Relation.PRODUCE, _expression_name(statement.value), scope))
        if kind is EntityKind.RESOURCE:
            resources.append(ResourceFact(statement.name, statement.type_name, scope))
            relations.append(RelationFact(scope, Relation.CREATE, statement.name, scope))
        _visit_expression(statement.value, scope, relations, effects, capabilities)
    elif isinstance(statement, Assign):
        target = _expression_name(statement.target)
        relations.append(RelationFact(target, Relation.WRITE, _expression_name(statement.value), scope))
        _visit_expression(statement.value, scope, relations, effects, capabilities)
    elif isinstance(statement, Call):
        _visit_call(statement, scope, relations, effects, capabilities)
    elif isinstance(statement, Return):
        relations.append(RelationFact(_expression_name(statement.value), Relation.PRODUCE, "return", scope))
        _visit_expression(statement.value, scope, relations, effects, capabilities)
    elif isinstance(statement, If):
        _visit_expression(statement.condition, scope, relations, effects, capabilities)
        for nested in statement.then_body:
            _visit_statement(nested, scope, entities, relations, resources, effects, capabilities)
        for nested in statement.else_body:
            _visit_statement(nested, scope, entities, relations, resources, effects, capabilities)
    elif isinstance(statement, While):
        _visit_expression(statement.condition, scope, relations, effects, capabilities)
        for nested in statement.body:
            _visit_statement(nested, scope, entities, relations, resources, effects, capabilities)
    elif isinstance(statement, For):
        if statement.initializer is not None:
            _visit_statement(statement.initializer, scope, entities, relations, resources, effects, capabilities)
        _visit_expression(statement.condition, scope, relations, effects, capabilities)
        for nested in statement.body:
            _visit_statement(nested, scope, entities, relations, resources, effects, capabilities)
        if statement.update is not None:
            _visit_statement(statement.update, scope, entities, relations, resources, effects, capabilities)


def _visit_expression(expression, scope, relations, effects, capabilities) -> None:
    if isinstance(expression, Binary):
        _visit_expression(expression.left, scope, relations, effects, capabilities)
        _visit_expression(expression.right, scope, relations, effects, capabilities)
        relations.append(RelationFact(_expression_name(expression.left), Relation.TRANSFORM, _expression_name(expression.right), scope))
    elif isinstance(expression, Unary):
        _visit_expression(expression.operand, scope, relations, effects, capabilities)
    elif isinstance(expression, (Index, FieldAccess)):
        _visit_expression(expression.target, scope, relations, effects, capabilities)
        if isinstance(expression, Index):
            _visit_expression(expression.index, scope, relations, effects, capabilities)
    elif isinstance(expression, StructLiteral):
        for _, value in expression.fields:
            _visit_expression(value, scope, relations, effects, capabilities)
    elif isinstance(expression, Call):
        _visit_call(expression, scope, relations, effects, capabilities)


def _visit_call(call: Call, scope, relations, effects, capabilities) -> None:
    relations.append(RelationFact(scope, Relation.CALL, call.name, scope))
    for argument in call.arguments:
        relations.append(RelationFact(_expression_name(argument), Relation.CONSUME, call.name, scope))
        _visit_expression(argument, scope, relations, effects, capabilities)

    if call.name in {"release", "close", "free", "drop"}:
        for argument in call.arguments:
            relations.append(RelationFact(_expression_name(argument), Relation.RELEASE, call.name, scope))

    effect_name, capability = _CALL_EFFECTS.get(call.name, (f"call.{call.name}", None))
    if effect_name:
        effects.append(EffectFact(effect_name, scope, call.name))
    if capability:
        capabilities.add(capability)


def _propagate_call_effects(program: Program, effects: list[EffectFact]) -> list[EffectFact]:
    by_function: dict[str, set[str]] = {}
    for effect in effects:
        by_function.setdefault(effect.scope, set()).add(effect.name)

    changed = True
    while changed:
        changed = False
        for function in program.functions:
            direct = by_function.setdefault(function.name, set())
            called = _called_functions(function)
            for target in called:
                for effect_name in by_function.get(target, ()):
                    if effect_name not in direct:
                        direct.add(effect_name)
                        changed = True

    result = [effect for effect in effects if not effect.transitive]
    for scope, names in by_function.items():
        existing = {effect.name for effect in result if effect.scope == scope}
        for name in sorted(names - existing):
            result.append(EffectFact(name, scope, f"call-graph:{scope}", True))
    return result


def _called_functions(function: Function) -> set[str]:
    called: set[str] = set()

    def walk(statement) -> None:
        if isinstance(statement, Call):
            if statement.name not in _CALL_EFFECTS:
                called.add(statement.name)
        elif isinstance(statement, If):
            for item in statement.then_body + statement.else_body:
                walk(item)
        elif isinstance(statement, While):
            for item in statement.body:
                walk(item)
        elif isinstance(statement, For):
            if statement.initializer:
                walk(statement.initializer)
            for item in statement.body:
                walk(item)
            if statement.update:
                walk(statement.update)

    for statement in function.body:
        walk(statement)
    return called


def _looks_like_resource(name: str, type_name: str | None) -> bool:
    haystack = f"{name} {type_name or ''}".lower()
    markers = ("resource", "file", "socket", "device", "buffer", "handle", "connection", "process", "thread", "gpu")
    return any(marker in haystack for marker in markers)


def _resource_argument_name(expression) -> str:
    if isinstance(expression, Variable):
        return expression.name
    if isinstance(expression, FieldAccess):
        return _resource_argument_name(expression.target)
    if isinstance(expression, Index):
        return _resource_argument_name(expression.target)
    return _expression_name(expression)


def _expression_name(expression) -> str:
    if isinstance(expression, Variable):
        return expression.name
    if isinstance(expression, FieldAccess):
        return f"{_expression_name(expression.target)}.{expression.field}"
    if isinstance(expression, Index):
        return f"{_expression_name(expression.target)}[]"
    if isinstance(expression, Call):
        return f"{expression.name}()"
    return "<expression>"
