# AXIOM SYSTEMS

AXIOM SYSTEMS es un ecosistema tecnológico construido por fases. La base actual ya no es solo un concepto: el proyecto tiene un bootstrap en Python y un backend nativo en Rust funcionando como una primera implementación de referencia y de ejecución real.

## Estado actual

La fase inicial ya ha dejado estable la identidad técnica y la arquitectura mínima del lenguaje:

- Bootstrap en Python para iterar rápido y validar comportamiento.
- Backend nativo en Rust dentro de [native](native).
- Pipeline funcional: lexer -> parser -> análisis semántico -> IR -> VM -> artefactos `.axm`.
- Soporte para tipos base, funciones, parámetros, retornos, arrays, indexación y asignación por índice.
- Bootstrap Python y backend Rust con una cobertura funcional cada vez más alineada.
- Validación automatizada en Rust y Python.

## Arquitectura actual

### Python bootstrap

La referencia de comportamiento vive en [src/axiom](src/axiom). Aquí se mantienen:

- lexer
- parser
- semántica
- runtime
- CLI

Esto sirve como base comparativa frente al backend nativo.

El bootstrap también dispone de un IR portable `.air` con instrucciones para:

- `LET`, `SET` y `PRINT`.
- bloques recursivos `IF`/`ELSE`, `WHILE` y `FOR`.
- control de bucle `BREAK` y `CONTINUE`.
- funciones, llamadas y retornos.
- arrays, indexación y expresiones aritméticas, comparativas o flotantes.

### Backend nativo

El núcleo nativo está en [native](native), con módulos clave como:

- [native/src/parser.rs](native/src/parser.rs): análisis sintáctico y tipos de array postfix.
- [native/src/semantic.rs](native/src/semantic.rs): validación de tipos y reglas del lenguaje.
- [native/src/ir.rs](native/src/ir.rs): lowering a instrucciones de VM.
- [native/src/vm.rs](native/src/vm.rs): compilación, serialización, deserialización y ejecución de artefactos.
- [native/src/runtime.rs](native/src/runtime.rs): runtime interpretado para pruebas y validación del comportamiento.

## Funcionalidades ya soportadas

- Literales: enteros, flotantes, booleanos, cadenas.
- Literales de arrays y arrays anidados.
- Variables y reasignaciones simples.
- Funciones con parámetros y retorno.
- Tipos explícitos, incluyendo `Int`, `Float` y arrays: `Int[]`, `Bool[]`, etc.
- Indexación de arrays, incluso encadenada: `values[1]`, `matrix[0][1]`.
- Asignación a elementos de arrays: `values[1] = 99`.
- Asignación anidada a arrays: `matrix[1][0] = 99`.
- Comentarios de línea con `//`.
- Expresiones agrupadas con paréntesis: `(2 + 3) * 4`.
- Negación unaria de enteros y flotantes: `-value`.
- Comparaciones: `>`, `>=`, `<`, `<=`, `==` y `!=`.
- Operadores aritméticos, lógicos y precedencia de expresiones.
- Control de flujo: `if`, `else if`, `else`, `while`, `for`, `break` y `continue`.
- Artefactos compilados `.axm` con serialización y ejecución posterior.
- Ejemplos reales de ramificación: `else if` encadenado con valores y condiciones.

## Validación actual

La base del proyecto ya está verificada con pruebas reales:

- Rust: 50 tests internos y 12 tests CLI pasando.
- Python: 43 tests pasando.
- `git diff --check` sin errores en la última validación.

## Desarrollo

```bash
# Python bootstrap
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
python -m pytest

# Native backend
cargo test --manifest-path native/Cargo.toml
cargo build --manifest-path native/Cargo.toml
native/target/debug/axiom check examples/hello.ax
native/target/debug/axiom build examples/hello.ax
native/target/debug/axiom run examples/hello.ax
```

## Roadmap actual

El roadmap completo sigue en [Documento sin título.txt](Documento%20sin%20t%C3%ADtulo.txt), que continúa siendo la fuente de dirección del proyecto. El trabajo actual está consolidando el lenguaje base y manteniendo Python como bootstrap mientras Rust avanza como implementación nativa.

Los siguientes movimientos razonables son:

- completar la paridad entre el IR Python `.air` y el backend nativo
- mejorar la calidad de errores semánticos del CLI con línea/columna reales
- ampliar las pruebas end-to-end de build, artefactos y funciones
- avanzar hacia la siguiente capa del lenguaje: estructuras, objetos, enums y tipos más ricos
- seguir usando Python como referencia mientras Rust consolida la implementación principal

## Principios

- Construir la mínima pieza que habilite la siguiente.
- Mantener interfaces pequeñas y comprobables.
- Priorizar claridad, seguridad y builds reproducibles.
- Medir antes de mover componentes críticos a una implementación nativa.

## Licencia

AXIOM SYSTEMS se distribuye bajo la licencia [MIT](LICENSE).