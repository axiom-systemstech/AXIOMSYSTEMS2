from axiom.parser import parse
from axiom.semantic import analyze
from axiom.ir import lower
from axiom.runtime import execute


def test_canonical_english_hello(capsys):
    program = parse('show("Hello AXIOM")')
    analyze(program)
    execute(lower(program))
    assert capsys.readouterr().out == "Hello AXIOM\n"


def test_canonical_if_else(capsys):
    program = parse(
        'age = 20\n'
        'if age >= 18:\n'
        '    show("Adult")\n'
        'else:\n'
        '    show("Minor")\n'
    )
    analyze(program)
    execute(lower(program))
    assert capsys.readouterr().out == "Adult\n"


def test_canonical_repeat(capsys):
    program = parse(
        'values = [1, 2, 3]\n'
        'repeat value in values:\n'
        '    show(value)\n'
    )
    analyze(program)
    execute(lower(program))
    assert capsys.readouterr().out == "1\n2\n3\n"


def test_canonical_struct_and_typed_entity(capsys):
    program = parse(
        'user:\n'
        '    name: String\n'
        '    age: Int\n'
        '\n'
        'alex: user\n'
        '    name = "Alex"\n'
        '    age = 25\n'
        '\n'
        'show(alex.age)\n'
    )
    analyze(program)
    execute(lower(program))
    assert capsys.readouterr().out == "25\n"


def test_canonical_function_with_final_expression(capsys):
    program = parse(
        'sum(a, b):\n'
        '    a + b\n'
        '\n'
        'show(sum(20, 22))\n'
    )
    analyze(program)
    execute(lower(program))
    assert capsys.readouterr().out == "42\n"
