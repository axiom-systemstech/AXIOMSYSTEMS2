from axiom.cli import main
from axiom.ast import BooleanLiteral, Call, Function, IntegerLiteral, Program, StringLiteral
from axiom.ast import Binary
from axiom.ast import Variable
from axiom.ir import IRProgram, IfInstruction, LetInstruction, PrintInstruction, SetInstruction, WhileInstruction, lower
from axiom.lexer import LexError, TokenKind, lex
from axiom.parser import ParseError, parse
from axiom.runtime import execute
from axiom.semantic import SemanticError, analyze


def test_doctor_reports_environment(capsys):
    assert main(["doctor"]) == 0
    output = capsys.readouterr().out
    assert "axiom 0.1.0" in output
    assert "python " in output
    assert "platform " in output


def test_check_accepts_axiom_file(tmp_path, capsys):
    source = tmp_path / "hello.ax"
    source.write_text('fn main() { print("Hello AXIOM") }', encoding="utf-8")
    assert main(["check", str(source)]) == 0
    assert f"ok: {source}" in capsys.readouterr().out


def test_run_executes_program_through_ir(tmp_path, capsys):
    source = tmp_path / "functions.ax"
    source.write_text(
        "fn add(a: Int, b: Int) -> Int { return a + b } "
        "fn main() { print(add(20, 22)) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "42\n"


def test_lexer_tokenizes_minimal_program():
    tokens = lex('fn main() { print("Hello AXIOM"); }')
    assert [token.kind for token in tokens] == [
        TokenKind.FN,
        TokenKind.IDENTIFIER,
        TokenKind.LPAREN,
        TokenKind.RPAREN,
        TokenKind.LBRACE,
        TokenKind.IDENTIFIER,
        TokenKind.LPAREN,
        TokenKind.STRING,
        TokenKind.RPAREN,
        TokenKind.SEMICOLON,
        TokenKind.RBRACE,
        TokenKind.EOF,
    ]
    assert tokens[5].lexeme == "print"
    assert tokens[7].line == 1


def test_parser_supports_integer_and_boolean_literals():
    program = parse("fn main() { print(42); print(true) }")
    assert program.functions[0].body == [
        Call("print", [IntegerLiteral(42)]),
        Call("print", [BooleanLiteral(True)]),
    ]


def test_lexer_reports_invalid_character_position():
    try:
        lex("fn main() { @ }")
    except LexError as error:
        assert str(error) == "unexpected character '@' at 1:13"
    else:
        raise AssertionError("expected LexError")


def test_lexer_skips_line_comments_and_preserves_division():
    tokens = lex('// ignored\nprint("http://axiom"); print(6 / 2)')
    assert [token.kind for token in tokens].count(TokenKind.SLASH) == 1
    assert tokens[0].lexeme == "print"
    assert tokens[0].line == 2
    assert tokens[0].column == 1


def test_parser_builds_ast_for_minimal_program():
    program = parse('fn main() { print("Hello AXIOM"); }')
    assert program == Program(
        [Function("main", [Call("print", [StringLiteral("Hello AXIOM")])])]
    )


def test_parser_reports_missing_function_body():
    try:
        parse("fn main()")
    except ParseError as error:
        assert str(error) == "expected '{' at 1:10"
    else:
        raise AssertionError("expected ParseError")


def test_semantic_analysis_accepts_minimal_program():
    analyze(parse('fn main() { print("Hello AXIOM") }'))


def test_semantic_analysis_requires_main():
    try:
        analyze(parse('fn start() { print("Hello AXIOM") }'))
    except SemanticError as error:
        assert str(error) == "program must define 'main'"
    else:
        raise AssertionError("expected SemanticError")


def test_semantic_analysis_rejects_unknown_call():
    try:
        analyze(parse('fn main() { display("Hello AXIOM") }'))
    except SemanticError as error:
        assert str(error) == "unknown function 'display'"
    else:
        raise AssertionError("expected SemanticError")


def test_ir_lowers_main_print_call():
    program = parse('fn main() { print("Hello AXIOM") }')
    assert lower(program) == IRProgram([PrintInstruction(StringLiteral("Hello AXIOM"))])
    assert lower(program).render() == "AXIOM-IR 0.1\nPRINT 'Hello AXIOM'\n"


def test_ir_renders_arrays_indexing_and_comparisons():
    program = parse("fn main() { let values: Int[] = [10, 20, 30]; print(values[1]); print(2 <= 3) }")
    assert lower(program).render() == (
        "AXIOM-IR 0.1\n"
        "LET values = [10, 20, 30]\n"
        "PRINT values[1]\n"
        "PRINT 2 <= 3\n"
    )


def test_ir_renders_assignments():
    program = parse(
        "fn main() { let values: Int[] = [10, 20]; values[1] = 99; print(values[1]) }"
    )
    assert lower(program).render() == (
        "AXIOM-IR 0.1\n"
        "LET values = [10, 20]\n"
        "SET values[1] = 99\n"
        "PRINT values[1]\n"
    )


def test_ir_runtime_executes_set_instruction():
    output = []
    execute(
        IRProgram(
            [
                SetInstruction(Variable("value"), IntegerLiteral(42)),
                PrintInstruction(Variable("value")),
            ]
        ),
        output.append,
    )
    assert output == ["42"]


def test_ir_renders_and_executes_if_else():
    program = parse('fn main() { if 2 < 3 { print("yes") } else { print("no") } }')
    assert lower(program).render() == (
        "AXIOM-IR 0.1\n"
        "IF 2 < 3\n"
        "  PRINT 'yes'\n"
        "ELSE\n"
        "  PRINT 'no'\n"
        "END\n"
    )
    output = []
    execute(lower(program), output.append)
    assert output == ["yes"]


def test_ir_if_branch_can_mutate_shared_state():
    output = []
    execute(
        IRProgram(
            [
                LetInstruction("value", IntegerLiteral(1)),
                IfInstruction(
                    BooleanLiteral(True),
                    [SetInstruction(Variable("value"), IntegerLiteral(2))],
                    [],
                ),
                PrintInstruction(Variable("value")),
            ]
        ),
        output.append,
    )
    assert output == ["2"]


def test_ir_renders_and_executes_while_loop():
    program = parse("fn main() { let count: Int = 0; while count < 3 { print(count); count = count + 1 } }")
    assert lower(program).render() == (
        "AXIOM-IR 0.1\n"
        "LET count = 0\n"
        "WHILE count < 3\n"
        "  PRINT count\n"
        "  SET count = count + 1\n"
        "END\n"
    )
    output = []
    execute(lower(program), output.append)
    assert output == ["0", "1", "2"]


def test_ir_while_body_can_contain_if():
    output = []
    execute(
        IRProgram(
            [
                LetInstruction("count", IntegerLiteral(0)),
                WhileInstruction(
                    Binary(Variable("count"), "<", IntegerLiteral(2)),
                    [
                        IfInstruction(
                            Binary(Variable("count"), "==", IntegerLiteral(0)),
                            [PrintInstruction(StringLiteral("first"))],
                            [],
                        ),
                        SetInstruction(
                            Variable("count"),
                            Binary(Variable("count"), "+", IntegerLiteral(1)),
                        ),
                    ],
                ),
            ]
        ),
        output.append,
    )
    assert output == ["first"]


def test_ir_lowers_and_executes_function_return():
    program = parse(
        "fn add(a: Int, b: Int) -> Int { return a + b } "
        "fn main() { print(add(20, 22)) }"
    )
    assert lower(program).render() == (
        "AXIOM-IR 0.1\n"
        "FUNCTION main()\n"
        "  PRINT add(20, 22)\n"
        "END FUNCTION\n"
        "FUNCTION add(a, b)\n"
        "  RETURN a + b\n"
        "END FUNCTION\n"
    )
    output = []
    execute(lower(program), output.append)
    assert output == ["42"]


def test_build_writes_python_ir_for_arrays(tmp_path, capsys):
    source = tmp_path / "arrays.ax"
    output = tmp_path / "arrays.air"
    source.write_text("fn main() { let values: Int[] = [10, 20]; values[1] = 99; print(values[1]) }", encoding="utf-8")

    assert main(["build", str(source), "--output", str(output)]) == 0
    assert output.read_text(encoding="utf-8") == (
        "AXIOM-IR 0.1\nLET values = [10, 20]\nSET values[1] = 99\nPRINT values[1]\n"
    )
    assert f"built: {output}" in capsys.readouterr().out


def test_runtime_executes_print_instruction():
    output = []
    execute(IRProgram([PrintInstruction("Hello AXIOM")]), output.append)
    assert output == ["Hello AXIOM"]


def test_runtime_executes_variable_and_addition(tmp_path, capsys):
    source = tmp_path / "math.ax"
    source.write_text("fn main() { let total: Int = 20 + 22; print(total) }", encoding="utf-8")
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "42\n"


def test_runtime_executes_function_with_parameters_and_return(tmp_path, capsys):
    source = tmp_path / "functions.ax"
    source.write_text(
        "fn add(a: Int, b: Int) -> Int { return a + b }\n"
        "fn main() { print(add(20, 22)) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "42\n"


def test_user_defined_function_is_not_hard_coded(tmp_path, capsys):
    source = tmp_path / "multiply.ax"
    source.write_text(
        "fn multiply(a: Int, b: Int) -> Int { return a + a + b }\n"
        "fn main() { print(multiply(10, 20)) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "40\n"


def test_semantic_analysis_rejects_unknown_type():
    try:
        analyze(parse("fn main(value: Any) { print(value) }"))
    except SemanticError as error:
        assert str(error) == "function 'main' uses an unknown parameter type"
    else:
        raise AssertionError("expected SemanticError")


def test_semantic_analysis_requires_declared_return():
    try:
        analyze(parse("fn answer() -> Int { print(42) } fn main() { print(42) }"))
    except SemanticError as error:
        assert str(error) == "function 'answer' must return Int"
    else:
        raise AssertionError("expected SemanticError")


def test_runtime_executes_while_and_assignment(tmp_path, capsys):
    source = tmp_path / "loop.ax"
    source.write_text(
        "fn main() { let count: Int = 0; while count == 0 { print(count); count = count + 1 } }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "0\n"


def test_runtime_respects_operator_precedence(tmp_path, capsys):
    source = tmp_path / "operators.ax"
    source.write_text(
        "fn main() { print(2 + 3 * 4); print(!false && true); print(-(3 + 2)) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "14\ntrue\n-5\n"


def test_runtime_executes_arrays_and_indexed_assignment(tmp_path, capsys):
    source = tmp_path / "arrays.ax"
    source.write_text(
        "fn main() { let values: Int[] = [10, 20, 30]; values[1] = 99; print(values[1]) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "99\n"


def test_semantic_analysis_rejects_heterogeneous_arrays():
    try:
        analyze(parse("fn main() { print([1, true]) }"))
    except SemanticError as error:
        assert str(error) == "array elements must share the same type"
    else:
        raise AssertionError("expected SemanticError")


def test_runtime_executes_nested_array_assignment(tmp_path, capsys):
    source = tmp_path / "nested_arrays.ax"
    source.write_text(
        "fn main() { let matrix: Int[][] = [[10, 20], [30, 40]]; matrix[1][0] = 99; print(matrix[1][0]) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "99\n"


def test_runtime_executes_all_integer_comparisons(tmp_path, capsys):
    source = tmp_path / "comparisons.ax"
    source.write_text(
        "fn main() { if 2 != 3 { print(1) } if 2 < 3 { print(2) } if 3 <= 3 { print(3) } if 3 >= 3 { print(4) } }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "1\n2\n3\n4\n"


def test_runtime_executes_string_concatenation(tmp_path, capsys):
    source = tmp_path / "strings.ax"
    source.write_text(
        'fn main() { let greeting: String = "Hello"; let name: String = "AXIOM"; print(greeting + " " + name) }',
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "Hello AXIOM\n"


def test_runtime_executes_for_loop(tmp_path, capsys):
    source = tmp_path / "for.ax"
    source.write_text(
        'fn main() { for (let i: Int = 0; i < 3; i = i + 1) { print(i) } }',
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "0\n1\n2\n"


def test_runtime_executes_else_if_chains(tmp_path, capsys):
    source = tmp_path / "elif.ax"
    source.write_text(
        "fn main() { if false { print(\"no\") } else if 2 < 3 { print(\"yes\") } else { print(\"nope\") } }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "yes\n"


def test_runtime_executes_break_and_continue(tmp_path, capsys):
    source = tmp_path / "loop_control.ax"
    source.write_text(
        'fn main() { for (let i: Int = 0; i < 5; i = i + 1) { if i == 2 { continue } if i == 4 { break } print(i) } }',
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "0\n1\n3\n"