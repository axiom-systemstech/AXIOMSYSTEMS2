from pathlib import Path
import zipfile

import pytest
from axiom.cli import main
from axiom.ast import BooleanLiteral, Call, Function, IntegerLiteral, Program, StringLiteral
from axiom.ast import Binary
from axiom.ast import Variable
from axiom.ir import IRProgram, IfInstruction, LetInstruction, PrintInstruction, SetInstruction, WhileInstruction, lower
from axiom.lexer import LexError, TokenKind, lex
from axiom.parser import ParseError, parse
from axiom.runtime import execute, execute_program
from axiom.semantic import SemanticError, analyze
from axiom.studio import check_source, complete_source, debug_source, documentation_index, git_diff, package_info, package_workspace, profile_source, run_source, run_workspace_tests, terminal_command, run_source_file, visual_ir, workspace_info


def test_new_creates_project(tmp_path, capsys):
    project = tmp_path / "hello"
    assert main(["new", str(project)]) == 0
    assert "created:" in capsys.readouterr().out
    assert (project / "axiom.toml").exists()
    assert (project / "src/main.ax").read_text(encoding="utf-8") == 'show("Hello AXIOM")\n'


def test_test_command_runs_axiom_sources(tmp_path, capsys):
    tests_path = tmp_path / "tests"
    tests_path.mkdir()
    source = tests_path / "hello.ax"
    source.write_text('fn main() { print("ok") }', encoding="utf-8")
    assert main(["test", str(tests_path)]) == 0
    assert "test ok: 1 source file(s)" in capsys.readouterr().out


def test_test_command_runs_reproducibility_gate(tmp_path, capsys):
    fixture = tmp_path / "tests" / "fixtures"
    fixture.mkdir(parents=True)
    source = fixture / "reproducibility.ax"
    source.write_text('fn main() { print("stable") }', encoding="utf-8")
    assert main(["test", "reproducible"]) == 0
    output = capsys.readouterr().out
    assert "[PASS] reproducible-build: identical artifacts" in output


def test_test_command_exposes_independence_boundary(capsys):
    assert main(["test", "independence"]) == 1
    output = capsys.readouterr().out
    assert "[FAIL] python-independence" in output
    assert "[FAIL] rust-independence" in output


def test_test_command_checks_bootstrap_presence(capsys):
    assert main(["test", "bootstrap"]) == 0
    assert "[PASS] bootstrap: AXIOM-written bootstrap stages are present" in capsys.readouterr().out


def test_package_creates_archive(tmp_path, monkeypatch, capsys):
    monkeypatch.chdir(tmp_path)
    (tmp_path / "axiom.toml").write_text(
        '[package]\nname = "demo"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    output = tmp_path / "demo.axpkg"
    assert main(["package", "-o", str(output)]) == 0
    assert "packaged:" in capsys.readouterr().out
    assert output.exists()
    with zipfile.ZipFile(output) as archive:
        names = set(archive.namelist())
    assert "axiom.toml" in names
    assert all(not name.startswith(".git/") for name in names)


def test_add_package_from_local_registry(tmp_path, monkeypatch, capsys):
    registry = tmp_path / "registry"
    package = registry / "networking"
    package.mkdir(parents=True)
    (package / "axiom.toml").write_text(
        '[package]\nname = "networking"\nversion = "0.1.0"\n',
        encoding="utf-8",
    )
    project = tmp_path / "project"
    project.mkdir()
    monkeypatch.chdir(project)

    assert main(["add", "networking", "--registry", str(registry)]) == 0
    assert "added: networking 0.1.0" in capsys.readouterr().out
    assert (project / "vendor/networking/axiom.toml").exists()
    assert 'networking = "0.1.0"' in (project / "axiom.toml").read_text(encoding="utf-8")
    assert '"networking": {' in (project / "axiom.lock").read_text(encoding="utf-8")


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


def test_parser_builds_struct_literal_and_field_access():
    program = parse(
        "struct Point { x: Int, y: Int } fn main() { let point: Point = Point { x: 10, y: 20 }; print(point.x) }"
    )
    assert program.structs[0].name == "Point"
    assert program.structs[0].fields[0].name == "x"


def test_runtime_executes_standard_library(capsys):
    source = 'fn main() { print(len("axiom")); print(abs(-7)); print(min(3, 5)); print(max(3.0, 5.0)) }'
    program = parse(source)
    analyze(program)
    execute_program(program)
    assert capsys.readouterr().out == "5\n7\n3\n5.0\n"


def test_semantic_rejects_invalid_standard_library_call():
    program = parse("fn main() { print(len(1)) }")
    with pytest.raises(SemanticError, match="len expects a String or array"):
        analyze(program)


def test_runtime_executes_struct_field_assignment(tmp_path, capsys):
    source = tmp_path / "struct_assignment.ax"
    source.write_text(
        "struct Point { x: Int, y: Int } fn main() { let point: Point = Point { x: 10, y: 20 }; point.x = 42; print(point.x) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "42\n"


def test_semantic_analysis_rejects_unknown_struct_field_assignment():
    try:
        analyze(parse("struct Point { x: Int } fn main() { let point: Point = Point { x: 10 }; point.y = 42 }"))
    except SemanticError as error:
        assert str(error) == "unknown field 'y'"
    else:
        raise AssertionError("expected SemanticError")


def test_runtime_executes_struct_field_access(tmp_path, capsys):
    source = tmp_path / "structs.ax"
    source.write_text(
        "struct Point { x: Int, y: Int } fn main() { let point: Point = Point { x: 10, y: 20 }; print(point.x); print(point.y) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "10\n20\n"


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


def test_semantic_analysis_rejects_loop_control_outside_loop():
    for keyword in ("break", "continue"):
        try:
            analyze(parse(f"fn main() {{ if true {{ {keyword} }} }}"))
        except SemanticError as error:
            assert str(error) == f"{keyword} must be inside a loop"
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


def test_runtime_executes_float_arithmetic(tmp_path, capsys):
    source = tmp_path / "floats.ax"
    source.write_text(
        "fn main() { let value: Float = 1.5 + 2.5; print(value); print(value / 2.0) }",
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "4.0\n2.0\n"


def test_runtime_executes_integer_remainder(tmp_path, capsys):
    source = tmp_path / "remainder.ax"
    source.write_text("fn main() { print(17 % 5) }", encoding="utf-8")
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "2\n"


def test_runtime_short_circuits_logical_operators(tmp_path, capsys):
    source = tmp_path / "short_circuit.ax"
    source.write_text(
        'fn main() { if false && 1 / 0 == 0 { print("bad") } if true || 1 / 0 == 0 { print("ok") } }',
        encoding="utf-8",
    )
    assert main(["run", str(source)]) == 0
    assert capsys.readouterr().out == "ok\n"


def test_runtime_reports_float_division_by_zero(tmp_path, capsys):
    source = tmp_path / "float_zero.ax"
    source.write_text("fn main() { print(1.0 / 0.0) }", encoding="utf-8")
    assert main(["run", str(source)]) == 1
    assert capsys.readouterr().err == "error: division by zero\n"


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

def test_studio_checks_source_and_reports_diagnostics():
    assert check_source('fn main() { print("ok") }')["ok"] is True
    result = check_source("fn main() { display(1) }")
    assert result["ok"] is False
    assert result["diagnostics"][0]["message"] == "unknown function 'display'"


def test_studio_completes_language_symbols():
    assert "print" in complete_source("fn main() { pri", "pri")
    assert "Int" in complete_source("fn main() { let value: I", "I")
    assert "main" in complete_source("fn main() { ma", "ma")


def test_studio_reports_workspace_sources(tmp_path):
    (tmp_path / "axiom.toml").write_text("[package]\nname = \"demo\"\n", encoding="utf-8")
    (tmp_path / "src").mkdir()
    (tmp_path / "src/main.ax").write_text("fn main() {}", encoding="utf-8")
    info = workspace_info(tmp_path)
    assert info["manifest"] is True
    assert info["sources"] == ["src/main.ax"]


def test_studio_runs_source_and_captures_output():
    result = run_source('fn main() { print("studio") }')
    assert result == {"ok": True, "diagnostics": [], "output": "studio\n"}


def test_studio_tests_workspace(tmp_path):
    (tmp_path / "one.ax").write_text('fn main() { print(1) }', encoding="utf-8")
    result = run_workspace_tests(tmp_path)
    assert result["ok"] is True
    assert result["count"] == 1


def test_studio_reports_git_status(tmp_path):
    from axiom.studio import git_status
    result = git_status(tmp_path)
    assert result["ok"] is False


def test_studio_packages_workspace(tmp_path):
    (tmp_path / "axiom.toml").write_text('[package]\nname = "demo"\nversion = "0.1.0"\n', encoding="utf-8")
    (tmp_path / "src").mkdir()
    (tmp_path / "src/main.ax").write_text('fn main() {}', encoding="utf-8")
    output = tmp_path / "demo.axpkg"
    result = package_workspace(tmp_path, output)
    assert result["ok"] is True
    assert output.exists()


def test_studio_debugger_traces_instructions_and_breakpoints():
    result = debug_source('fn main() { let x: Int = 1; print(x) }', [{"function": "main", "index": 0}])
    assert result["ok"] is True
    assert result["events"]
    assert result["breakpoints"][0]["index"] == 0


def test_studio_profiler_collects_function_and_instruction_metrics():
    result = profile_source('fn helper() -> Int { return 3 } fn main() { print(helper()) }')
    assert result["ok"] is True
    assert any(item["name"] == "helper" for item in result["functions"])
    assert result["instructions"]


def test_studio_tests_single_source(tmp_path):
    source = tmp_path / "main.ax"
    source.write_text('fn main() { print("ok") }', encoding="utf-8")
    result = run_source_file(tmp_path, "main.ax")
    assert result["ok"] is True


def test_studio_package_info_and_docs(tmp_path):
    (tmp_path / "docs").mkdir()
    (tmp_path / "docs/guide.md").write_text("# Guide", encoding="utf-8")
    result = documentation_index(tmp_path)
    assert result["docs"] == ["docs/guide.md"]
    info = package_info(tmp_path)
    assert info["manifest"] is None


def test_studio_terminal_is_allowlisted(tmp_path):
    result = terminal_command(tmp_path, "git status")
    assert result["ok"] is False
    assert terminal_command(tmp_path, "rm -rf /")["ok"] is False


def test_studio_visual_ir_and_git_diff(tmp_path):
    result = visual_ir('fn main() { print("ir") }')
    assert result["ok"] is True
    assert "AXIOM-IR" in result["ir"]
    diff = git_diff(tmp_path)
    assert diff["ok"] is False


def test_core_type_model_is_structural():
    from axiom.types import Type, parse_type_name
    assert parse_type_name("Int").render() == "Int"
    assert parse_type_name("Int[][]").render() == "Int[][]"
    assert Type.array(Type.named("Point")).render() == "Point[]"


def test_module_imports_compile_as_one_program(tmp_path):
    from axiom.core import Compiler
    source_root = tmp_path / "src"
    source_root.mkdir()
    (source_root / "main.ax").write_text(
        "module main\nimport math\nfn main() { print(double(21)) }\n",
        encoding="utf-8",
    )
    (source_root / "math.ax").write_text(
        "module math\nfn double(value: Int) -> Int { return value + value }\n",
        encoding="utf-8",
    )
    compilation = Compiler().compile_project(tmp_path)
    assert [unit.module for unit in compilation.units] == ["main", "math"]
    assert "PRINT double(21)" in compilation.ir.render()


def test_module_import_missing_is_reported(tmp_path):
    from axiom.core import Compiler, CompilerError
    source_root = tmp_path / "src"
    source_root.mkdir()
    (source_root / "main.ax").write_text(
        "module main\nimport missing\nfn main() { print(1) }\n",
        encoding="utf-8",
    )
    with pytest.raises(CompilerError, match="module 'missing' not found"):
        Compiler().compile_project(tmp_path)


def test_module_import_cycle_is_reported(tmp_path):
    from axiom.core import Compiler, CompilerError
    source_root = tmp_path / "src"
    source_root.mkdir()
    (source_root / "main.ax").write_text(
        "module main\nimport util\nfn main() { print(1) }\n",
        encoding="utf-8",
    )
    (source_root / "util.ax").write_text(
        "module util\nimport main\nfn helper() { print(2) }\n",
        encoding="utf-8",
    )
    with pytest.raises(CompilerError, match="cyclic module import"):
        Compiler().compile_project(tmp_path)
