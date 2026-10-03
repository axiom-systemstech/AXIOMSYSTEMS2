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
