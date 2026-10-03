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
    SHARE = "SHARE"
    VIEW = "VIEW"


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


class ResourceState(str, Enum):
    ACTIVE = "ACTIVE"
    RELEASED = "RELEASED"
    MAYBE_RELEASED = "MAYBE_RELEASED"
    SHARED = "SHARED"


@dataclass(frozen=True)
class ResourceFlowFact:
    resource: str
    scope: str
    state: str
    consumers: tuple[str, ...] = ()
    aliases: tuple[str, ...] = ()
    views: tuple[str, ...] = ()
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

    _augment_structured_resources(program, entities, relations, resources)

    # Calls through user functions propagate their effects to callers.
    effects = _propagate_call_effects(program, effects)
    resource_flow, resource_diagnostics = _analyze_resource_flow(program, resources)
    for item in resource_flow:
        if item.state == "SHARED":
            for consumer in item.consumers:
                if consumer not in {"release", "close", "free", "drop"}:
                    relations.append(RelationFact(item.resource, Relation.SHARE, consumer, item.scope))
        for view in item.views:
            relations.append(RelationFact(item.resource, Relation.VIEW, view, item.scope))
            relations.append(RelationFact(item.resource, Relation.CONTAIN, view, item.scope))

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


def _augment_structured_resources(program: Program, entities, relations, resources) -> None:
    structs = {struct.name: struct for struct in program.structs}
    existing = {(resource.name, resource.created_in) for resource in resources}
    pending = [
        (entity.name, entity.type_name, entity.scope)
        for entity in entities
        if entity.kind is EntityKind.RESOURCE and entity.type_name in structs
    ]
    expanded: set[tuple[str, str, str]] = set()

    while pending:
        parent_name, type_name, scope = pending.pop()
        marker = (parent_name, type_name, scope)
        if marker in expanded:
            continue
        expanded.add(marker)
        struct = structs[type_name]
        for field in struct.fields:
            if not (
                _looks_like_resource(field.name, field.type_name)
                or _struct_contains_resource(field.type_name, structs)
            ):
                continue
            resource_name = f"{parent_name}.{field.name}"
            if (resource_name, scope) not in existing:
                resources.append(ResourceFact(resource_name, field.type_name, scope))
                relations.append(RelationFact(parent_name, Relation.CONTAIN, resource_name, scope))
                existing.add((resource_name, scope))
            if field.type_name in structs:
                pending.append((resource_name, field.type_name, scope))


def _struct_contains_resource(type_name: str | None, structs, seen: set[str] | None = None) -> bool:
    if type_name not in structs:
        return False
    seen = set() if seen is None else seen
    if type_name in seen:
        return False
    seen.add(type_name)
    return any(
        _looks_like_resource(field.name, field.type_name)
        or _struct_contains_resource(field.type_name, structs, seen)
        for field in structs[type_name].fields
    )


def _analyze_resource_flow(program: Program, resources: list[ResourceFact]) -> tuple[list[ResourceFlowFact], tuple[str, ...]]:
    functions = {function.name: function for function in program.functions}
    flow: list[ResourceFlowFact] = []
    diagnostics: list[str] = []

    for function in program.functions:
        roots = {item.name: item.name for item in resources if item.created_in == function.name}
        state = {name: ResourceState.ACTIVE for name in roots.values()}
        aliases = {name: set() for name in roots.values()}
        views = {name: set() for name in roots.values()}
        consumers = {name: [] for name in roots.values()}
        released_by = {}
        kinds = {name: "root" for name in roots}
        _walk_resource_block(
            function.body, function.name, functions, resources, roots, state, aliases, views,
            consumers, released_by, kinds, diagnostics, (function.name,),
        )
        for name in sorted(consumers):
            non_lifecycle = [item for item in consumers[name] if item not in {"release", "close", "free", "drop"}]
            if len(set(non_lifecycle)) > 1 and state[name] == ResourceState.ACTIVE:
                state[name] = ResourceState.SHARED
            flow.append(ResourceFlowFact(
                name, function.name, state[name].value, tuple(consumers[name]),
                tuple(sorted(aliases[name])), tuple(sorted(views[name])), released_by.get(name),
            ))
    return flow, tuple(diagnostics)


def _join_resource_state(left: ResourceState, right: ResourceState) -> ResourceState:
    if left == right:
        return left
    if ResourceState.SHARED in {left, right}:
        return ResourceState.SHARED
    return ResourceState.MAYBE_RELEASED


def _walk_resource_block(
    statements, scope, functions, resources, roots, state, aliases, views,
    consumers, released_by, kinds, diagnostics, call_stack,
):
    release_calls = {"release", "close", "free", "drop"}
    for statement in statements:
        if isinstance(statement, Let):
            if isinstance(statement.value, Call):
                returned = _apply_resource_call(
                    statement.value, scope, functions, resources, roots, state, aliases, views,
                    consumers, released_by, kinds, diagnostics, call_stack,
                )
                if returned is not None:
                    root, kind = returned
                    roots[statement.name] = root
                    kinds[statement.name] = kind
                    (views if kind == "view" else aliases)[root].add(statement.name)
                continue
            source = _resource_argument_name(statement.value, roots)
            if source in roots:
                root = roots[source]
                roots[statement.name] = root
                if isinstance(statement.value, Index):
                    kinds[statement.name] = "view"
                    views[root].update({_resource_view_name(statement.value, roots), statement.name})
                else:
                    kinds[statement.name] = "alias"
                    aliases[root].add(statement.name)
            continue

        if isinstance(statement, Assign) and isinstance(statement.target, Variable):
            source = _resource_argument_name(statement.value, roots)
            if source in roots:
                root = roots[source]
                roots[statement.target.name] = root
                if isinstance(statement.value, Index):
                    kinds[statement.target.name] = "view"
                    views[root].update({_expression_name(statement.value), statement.target.name})
                else:
                    kinds[statement.target.name] = "alias"
                    aliases[root].add(statement.target.name)
            continue

        if isinstance(statement, Return):
            source = _resource_argument_name(statement.value, roots)
            if source in roots:
                roots["__return__"] = roots[source]
                kinds["__return__"] = kinds.get(source, "root")
            continue

        if isinstance(statement, Call):
            _apply_resource_call(
                statement, scope, functions, resources, roots, state, aliases, views,
                consumers, released_by, kinds, diagnostics, call_stack,
            )
            continue

        if isinstance(statement, If):
            then_state, else_state = dict(state), dict(state)
            _walk_resource_block(
                statement.then_body, scope, functions, resources, dict(roots), then_state,
                aliases, views, consumers, released_by, dict(kinds), diagnostics, call_stack,
            )
            _walk_resource_block(
                statement.else_body, scope, functions, resources, dict(roots), else_state,
                aliases, views, consumers, released_by, dict(kinds), diagnostics, call_stack,
            )
            for root in state:
                state[root] = _join_resource_state(then_state[root], else_state[root])
            continue

        if isinstance(statement, While):
            before, loop_state = dict(state), dict(state)
            _walk_resource_block(
                statement.body, scope, functions, resources, dict(roots), loop_state,
                aliases, views, consumers, released_by, dict(kinds), diagnostics, call_stack,
            )
            for root in state:
                state[root] = _join_resource_state(before[root], loop_state[root])
            continue

        if isinstance(statement, For):
            if statement.initializer is not None:
                _walk_resource_block(
                    [statement.initializer], scope, functions, resources, roots, state,
                    aliases, views, consumers, released_by, kinds, diagnostics, call_stack,
                )
            before, loop_state = dict(state), dict(state)
            _walk_resource_block(
                statement.body, scope, functions, resources, dict(roots), loop_state,
                aliases, views, consumers, released_by, dict(kinds), diagnostics, call_stack,
            )
            for root in state:
                state[root] = _join_resource_state(before[root], loop_state[root])


def _apply_resource_call(
    call, scope, functions, resources, roots, state, aliases, views,
    consumers, released_by, kinds, diagnostics, call_stack,
):
    release_calls = {"release", "close", "free", "drop"}
    argument_roots = []
    for argument in call.arguments:
        name = _resource_argument_name(argument, roots)
        if name not in roots:
            continue
        root = roots[name]
        argument_roots.append((argument, root))
        if isinstance(argument, Index):
            views[root].add(_resource_view_name(argument, roots))
        if state[root] in {ResourceState.RELEASED, ResourceState.MAYBE_RELEASED}:
            diagnostics.append(f"resource '{root}' is used after release in '{scope}'")
            continue
        consumers[root].append(call.name)
        if call.name in release_calls:
            state[root] = ResourceState.RELEASED
            released_by[root] = call.name

    callee = functions.get(call.name)
    if callee is None or call.name in call_stack:
        if callee is not None and call.name in call_stack:
            for _, root in argument_roots:
                state[root] = _join_resource_state(state[root], ResourceState.MAYBE_RELEASED)
        return None

    resource_params = [p for p in callee.parameters if _looks_like_resource(p.name, p.type_name)]
    if not resource_params:
        return None

    callee_roots = {p.name: p.name for p in resource_params}
    for resource in resources:
        if resource.created_in != callee.name:
            continue
        if any(resource.name.startswith(p.name + ".") for p in resource_params):
            callee_roots[resource.name] = resource.name
    callee_state = {name: ResourceState.ACTIVE for name in callee_roots}
    callee_aliases = {name: set() for name in callee_roots}
    callee_views = {name: set() for name in callee_roots}
    callee_consumers = {name: [] for name in callee_roots}
    callee_released = {}
    callee_kinds = {name: "root" for name in callee_roots}
    callee_diagnostics = []
    _walk_resource_block(
        callee.body, callee.name, functions, resources, callee_roots, callee_state,
        callee_aliases, callee_views, callee_consumers, callee_released,
        callee_kinds, callee_diagnostics, call_stack + (callee.name,),
    )
    diagnostics.extend(callee_diagnostics)

    for index, parameter in enumerate(callee.parameters):
        if parameter.name not in callee_roots or index >= len(call.arguments):
            continue
        source = _resource_argument_name(call.arguments[index], roots)
        if source not in roots:
            continue
        root = roots[source]
        nested_state = callee_state[parameter.name]
        if nested_state == ResourceState.RELEASED:
            state[root] = ResourceState.RELEASED
            released_by[root] = callee_released.get(parameter.name, call.name)
        elif nested_state != ResourceState.ACTIVE:
            state[root] = _join_resource_state(state[root], nested_state)
        aliases[root].update(f"{call.name}.{item}" for item in callee_aliases[parameter.name])
        views[root].update(f"{call.name}.{item}" for item in callee_views[parameter.name])

        # Structured resource fields retain identity across the call boundary.
        prefix = parameter.name + "."
        for callee_root in callee_roots:
            if not callee_root.startswith(prefix):
                continue
            suffix = callee_root[len(prefix):]
            caller_root = f"{root}.{suffix}"
            if caller_root not in roots:
                continue
            nested_field_state = callee_state[callee_root]
            if nested_field_state == ResourceState.RELEASED:
                state[caller_root] = ResourceState.RELEASED
                released_by[caller_root] = callee_released.get(callee_root, call.name)
            elif nested_field_state != ResourceState.ACTIVE:
                state[caller_root] = _join_resource_state(state[caller_root], nested_field_state)
            aliases[caller_root].update(
                f"{call.name}.{item}" for item in callee_aliases[callee_root]
            )
            views[caller_root].update(
                f"{call.name}.{item}" for item in callee_views[callee_root]
            )

    returned = callee_roots.get("__return__")
    if returned is not None:
        for index, parameter in enumerate(callee.parameters):
            if parameter.name == returned and index < len(call.arguments):
                source = _resource_argument_name(call.arguments[index], roots)
                if source in roots:
                    return roots[source], "alias"
    return None


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


def _resource_view_name(expression, roots) -> str:
    if not isinstance(expression, Index):
        return _expression_name(expression)
    depth = 0
    target = expression
    while isinstance(target, Index):
        depth += 1
        target = target.target
    root = roots.get(_resource_argument_name(target, roots), _expression_name(target))
    return root + "[]" * depth


def _resource_argument_name(expression, roots=None) -> str:
    if isinstance(expression, Variable):
        return expression.name
    if isinstance(expression, FieldAccess):
        full_name = _expression_name(expression)
        if roots is not None and full_name in roots:
            return full_name
        return _resource_argument_name(expression.target, roots)
    if isinstance(expression, Index):
        return _resource_argument_name(expression.target, roots)
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
