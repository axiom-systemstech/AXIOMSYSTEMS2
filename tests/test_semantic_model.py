from axiom.parser import parse
from axiom.semantic import analyze
from axiom.semantic_model import EntityKind, Relation


def test_semantic_model_records_entities_relations_and_effects():
    program = parse(
        'show("hello")\n'
        'value = 2\n'
        'show(value)\n'
    )
    model = analyze(program)

    assert any(entity.name == "value" and entity.kind is EntityKind.VALUE for entity in model.entities)
    assert any(relation.relation is Relation.PRODUCE and relation.source == "value" for relation in model.relations)
    assert any(effect.name == "terminal.write" and effect.scope == "main" for effect in model.effects)
    assert "terminal.write" in model.capabilities


def test_semantic_model_propagates_function_effects():
    program = parse(
        'announce():\n'
        '    show("hello")\n'
        '\n'
        'announce()\n'
    )
    model = analyze(program)

    assert any(effect.name == "terminal.write" and effect.scope == "announce" for effect in model.effects)
    assert any(
        effect.name == "terminal.write"
        and effect.scope == "main"
        and effect.transitive
        for effect in model.effects
    )


def test_semantic_model_marks_resource_like_entities():
    program = parse(
        'Buffer:\n'
        '    size: Int\n'
        '\n'
        'buffer: Buffer\n'
        '    size = 4\n'
        '\n'
        'show(buffer.size)\n'
    )
    model = analyze(program)

    assert any(entity.name == "buffer" and entity.kind is EntityKind.RESOURCE for entity in model.entities)


def test_canonical_directives_become_semantic_constraints():
    from axiom.new_parser import parse_new

    program = parse_new(
        "needs:\n"
        "    terminal.write\n"
        "can:\n"
        "    terminal.write\n"
        "restrict:\n"
        "    execution = sequential\n"
        "prefer:\n"
        "    placement = local\n"
        "mode:\n"
        "    real_time\n"
        "prove:\n"
        "    deterministic\n"
        "show(\"hello\")\n"
    )
    model = analyze(program)

    assert model.requirements == ("terminal.write",)
    assert model.allowed_capabilities == ("terminal.write",)
    assert model.restrictions == ("execution = sequential",)
    assert model.preferences == ("placement = local",)
    assert model.modes == ("real_time",)
    assert model.contracts == ("deterministic",)
    assert not model.diagnostics


def test_capability_restriction_is_diagnostic():
    from axiom.new_parser import parse_new

    program = parse_new(
        "can:\n"
        "    filesystem.read\n"
        "show(\"hello\")\n"
    )
    model = analyze(program)
    assert "capability 'terminal.write' is not allowed by the program" in model.diagnostics


def test_resource_flow_tracks_release_and_consumers():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "show(buffer.value)\n"
        "release(buffer)\n"
    )
    model = analyze(program)
    flow = model.resource_flow[0]

    assert flow.resource == "buffer"
    assert flow.state == "RELEASED"
    assert flow.consumers == ("print", "release")
    assert flow.released_by == "release"
    assert any(
        relation.relation.value == "RELEASE"
        and relation.source == "buffer"
        for relation in model.relations
    )


def test_resource_flow_reports_use_after_release():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "release(buffer)\n"
        "show(buffer.value)\n"
    )
    model = analyze(program)

    assert "resource 'buffer' is used after release in 'main'" in model.diagnostics


def test_resource_flow_preserves_identity_through_alias():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "alias = buffer\n"
        "show(alias.value)\n"
        "release(alias)\n"
    )
    model = analyze(program)
    flow = next(item for item in model.resource_flow if item.resource == "buffer")

    assert flow.state == "RELEASED"
    assert flow.aliases == ("alias",)
    assert flow.released_by == "release"


def test_resource_flow_branch_join_is_conservative():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "if true:\n"
        "    release(buffer)\n"
        "show(buffer.value)\n"
    )
    model = analyze(program)
    flow = next(item for item in model.resource_flow if item.resource == "buffer")

    assert flow.state == "MAYBE_RELEASED"
    assert "resource 'buffer' is used after release in 'main'" in model.diagnostics


def test_resource_flow_marks_multiple_consumers_as_shared():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "inspect(buffer: Buffer):\n"
        "    show(buffer.value)\n"
        "consume(buffer: Buffer):\n"
        "    show(buffer.value)\n"
        "inspect(buffer)\n"
        "consume(buffer)\n"
    )
    model = analyze(program)
    flow = next(item for item in model.resource_flow if item.resource == "buffer" and item.scope == "main")

    assert flow.state == "SHARED"
    assert flow.consumers == ("inspect", "consume")


def test_resource_flow_emits_share_relations_for_multiple_consumers():
    from axiom.new_parser import parse_new

    program = parse_new(
        "Buffer:\n"
        "    value: Int\n"
        "inspect(buffer: Buffer):\n"
        "    show(buffer.value)\n"
        "consume(buffer: Buffer):\n"
        "    show(buffer.value)\n"
        "buffer: Buffer\n"
        "    value = 1\n"
        "inspect(buffer)\n"
        "consume(buffer)\n"
    )
    model = analyze(program)

    shares = [relation for relation in model.relations if relation.relation is Relation.SHARE]
    assert {(item.source, item.target, item.scope) for item in shares} == {
        ("buffer", "inspect", "main"),
        ("buffer", "consume", "main"),
    }


def test_resource_flow_tracks_index_as_view_of_resource():
    from axiom.new_parser import parse_new

    program = parse_new(
        "buffers = [1, 2]\n"
        "show(buffers[0])\n"
    )
    model = analyze(program)
    flow = next(item for item in model.resource_flow if item.resource == "buffers")

    assert flow.views == ("buffers[]",)
    assert any(
        relation.relation is Relation.VIEW
        and relation.source == "buffers"
        and relation.target == "buffers[]"
        for relation in model.relations
    )
