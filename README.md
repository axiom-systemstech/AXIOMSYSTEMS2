# AXIOM SYSTEMS

AXIOM SYSTEMS es un lenguaje y ecosistema tecnológico construido por fases.
El repositorio contiene dos implementaciones coordinadas:

- Un **bootstrap en Python**, usado como referencia rápida de comportamiento.
- Un **backend nativo en Rust**, usado para el compilador, la VM y los artefactos ejecutables.

La estrategia actual es pequeña y deliberada: cada capacidad debe quedar
probada en el bootstrap, portada a Rust y comprobada también desde el CLI antes
de abrir la siguiente capa del lenguaje.

## Estado actual

El núcleo del lenguaje ya tiene un pipeline funcional de extremo a extremo:

```text
AXIOM (.ax)
  -> lexer
  -> parser
  -> análisis semántico
  -> IR
  -> runtime / VM
  -> salida o artefacto .axm
```

El bootstrap Python genera IR portable `.air`. El backend Rust genera artefactos
`.axm`, los serializa, los deserializa y los ejecuta posteriormente.

## Capacidades implementadas

### Sintaxis y tipos

- Funciones, parámetros, retornos y llamadas.
- Tipos `Int`, `Float`, `Bool` y `String`.
- Arrays y arrays anidados: `Int[]`, `Float[][]`, etc.
- Structs con campos de tipos base, literales y acceso de lectura mediante punto.
- Comentarios de línea con `//`.
- Literales agrupados con paréntesis.

Ejemplo de structs:

```axiom
struct Point {
    x: Int
    y: Int
}

fn main() {
    let point: Point = Point { x: 10, y: 20 }
    print(point.x)
}
```

### Expresiones

- Aritmética: `+`, `-`, `*`, `/` y `%`.
- Operaciones homogéneas para `Int` y `Float`.
- Concatenación de `String` con `+`.
- Comparaciones: `>`, `>=`, `<`, `<=`, `==` y `!=`.
- Lógica booleana: `!`, `&&` y `||`.
- Cortocircuito lógico: el operando derecho solo se evalúa cuando es necesario.
- Negación unaria de enteros y flotantes.
- Indexación y asignación de arrays, incluida la indexación encadenada.

### Control de flujo

- `if`, `else if` y `else`.
- `while`.
- `for` con inicializador, condición y actualización.
- `break` y `continue` con validación semántica de contexto.
- Actualización correcta del `for` después de `continue`.

### Herramientas

- CLI Python: `check`, `build`, `run` y `doctor`.
- CLI Rust: `check`, `build` y `run`.
- Build y ejecución de artefactos `.axm`.
- Errores léxicos y sintácticos con línea y columna en el backend nativo.
- Errores semánticos para tipos incompatibles, retornos inválidos y control de flujo fuera de bucles.

## Arquitectura del repositorio

### Bootstrap Python

La implementación de referencia está en [src/axiom](src/axiom):

- [lexer.py](src/axiom/lexer.py): tokens, palabras reservadas y posiciones.
- [parser.py](src/axiom/parser.py): parser recursivo descendente.
- [ast.py](src/axiom/ast.py): nodos del lenguaje.
- [semantic.py](src/axiom/semantic.py): tipos y reglas semánticas.
- [ir.py](src/axiom/ir.py): lowering al IR `.air`.
- [runtime.py](src/axiom/runtime.py): ejecución del IR.
- [cli.py](src/axiom/cli.py): herramienta Python.

### Backend Rust

La implementación nativa está en [native](native):

- [native/src/lib.rs](native/src/lib.rs): lexer y tokens.
- [native/src/parser.rs](native/src/parser.rs): AST y parser nativo.
- [native/src/semantic.rs](native/src/semantic.rs): análisis semántico.
- [native/src/ir.rs](native/src/ir.rs): instrucciones de VM y lowering.
- [native/src/runtime.rs](native/src/runtime.rs): runtime interpretado de referencia para pruebas.
- [native/src/vm.rs](native/src/vm.rs): VM, artefactos `.axm`, serialización y ejecución.
- [native/src/main.rs](native/src/main.rs): CLI nativo.
- [native/tests/cli_errors.rs](native/tests/cli_errors.rs): pruebas CLI end-to-end.

## Comandos de desarrollo

### Python

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
python -m pytest
```

Uso rápido:

```bash
axiom doctor
axiom check examples/hello.ax
axiom build examples/hello.ax
axiom run examples/hello.ax
```

### Rust

```bash
cargo fmt --manifest-path native/Cargo.toml -- --check
cargo clippy --manifest-path native/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path native/Cargo.toml
cargo build --manifest-path native/Cargo.toml
```

Uso del CLI nativo:

```bash
native/target/debug/axiom check examples/hello.ax
native/target/debug/axiom build examples/hello.ax
native/target/debug/axiom run examples/hello.ax
```

## Validación actual

Estado comprobado en el último ciclo de desarrollo:

- Rust: **52 pruebas unitarias** pasando.
- Rust CLI: **13 pruebas end-to-end** pasando.
- Python: **45 pruebas** pasando.
- Clippy con `-D warnings`: limpio.
- Formato Rust y `git diff --check`: limpios.

## Próximos movimientos

Orden recomendado para continuar el proyecto:

1. Completar la semántica de structs en Rust y Python: validar campos desconocidos, duplicados y tipos de cada campo de forma uniforme.
2. Añadir asignación de campos, por ejemplo `point.x = 42`, y extenderla al IR y a los artefactos.
3. Añadir pruebas CLI de errores semánticos con mensajes y ubicaciones precisas.
4. Consolidar la paridad de `.air` y `.axm`, incluyendo una especificación versionada de ambos formatos.
5. Diseñar y portar la siguiente capa de tipos ricos: enums, optionals y manejo explícito de errores.
6. Automatizar formato, Clippy y las suites Python/Rust en CI.

El roadmap de visión completa está en [Documento sin título.txt](Documento%20sin%20t%C3%ADtulo.txt). La documentación específica de sintaxis está en [docs/axiom-language.md](docs/axiom-language.md).

## Principios del proyecto

- Construir la mínima pieza que habilite la siguiente.
- Mantener Python como referencia rápida y Rust como implementación nativa.
- No aceptar una feature sin pruebas de parser, semántica y ejecución cuando aplique.
- Mantener interfaces pequeñas, artefactos reproducibles y errores comprensibles.
- Portar capacidades por paridad, no por acumulación de implementaciones divergentes.

## Licencia

AXIOM SYSTEMS se distribuye bajo la licencia [MIT](LICENSE).