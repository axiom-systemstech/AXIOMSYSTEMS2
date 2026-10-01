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

- Rust: suite nativa completa pasando.
- Rust CLI: 15 pruebas end-to-end pasando.
- Workflow nativo de Fase 18: new/check/run/build/test/package/install validado.
- Clippy con `-D warnings`: limpio.
- Formato Rust y `git diff --check`: limpios.

## Estado actual

Las fases 1–18 del sub-roadmap del lenguaje están completadas. El binario nativo
es la referencia del toolchain: puede crear, comprobar, compilar, ejecutar,
probar, empaquetar, instalar y producir ejecutables standalone sin Python.

La documentación del workflow nativo está en docs/native-toolchain.md.
La superficie de comandos está en docs/cli.md.

El roadmap de visión completa está en [Documento sin título.txt](Documento%20sin%20t%C3%ADtulo.txt). La documentación específica de sintaxis está en [docs/axiom-language.md](docs/axiom-language.md). La especificación de los formatos intermedios está en [docs/ir-formats.md](docs/ir-formats.md). La primera capa de la librería estándar está en [docs/standard-library.md](docs/standard-library.md). El modelo inicial de paquetes está en [docs/package-manager.md](docs/package-manager.md). La superficie de comandos está en [docs/cli.md](docs/cli.md).

## Principios del proyecto

- Construir la mínima pieza que habilite la siguiente.
- Mantener el binario nativo como referencia del toolchain; Python queda como compatibilidad histórica.
- No aceptar una feature sin pruebas de parser, semántica y ejecución cuando aplique.
- Mantener interfaces pequeñas, artefactos reproducibles y errores comprensibles.
- Portar capacidades por paridad, no por acumulación de implementaciones divergentes.

## Licencia

AXIOM SYSTEMS se distribuye bajo la licencia [MIT](LICENSE).