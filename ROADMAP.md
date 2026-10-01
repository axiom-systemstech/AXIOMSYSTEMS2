# AXIOM Language Roadmap

> Roadmap vivo del lenguaje AXIOM y de su toolchain. El roadmap maestro de AXIOM SYSTEMS contiene además Engine, Cloud, AI, OS, hardware y el resto del ecosistema.

## Estado actual

- Fases completadas: 1–16.
- Fase actual: 17 — Self-hosting completo.
- Hito alcanzado: lexer, parser, AST estructurado y primera capa semántica ejecutándose desde AXIOM.
- Próximo hito: ampliar la semántica AXIOM hacia resolución de nombres, scopes y comprobación de expresiones.
- Después: 18 — Independencia completa del ecosistema.
- Objetivo de independencia: bootstrap en Fase 16, self-hosting en Fase 17 e independencia completa del toolchain en Fase 18.

## Fundamentos

- [x] ~~Fase 1 — Fundamentos del lenguaje~~: sintaxis, lexer, parser, AST y programas ejecutables.
- [x] ~~Fase 2 — Sistema semántico~~: tipos, variables, funciones, retornos y validación semántica.
- [x] ~~Fase 3 — Control de flujo~~: condicionales, bucles, `break`, `continue`, booleanos y comparaciones.
- [x] ~~Fase 4 — Tipos y estructuras de datos~~: arrays, indexación, strings, floats, aritmética y tipado explícito.
- [x] ~~Fase 5 — Structs y modelo de datos~~: structs, campos, acceso, asignación y validación.

## Compiler pipeline

- [x] ~~Fase 6 — AXIOM IR~~: IR, lowering AST→IR, ejecución, serialización y artefactos compilados.
- [x] ~~Fase 7 — Runtime / VM~~: VM, runtime, funciones, arrays, structs, control de flujo y errores.
- [x] ~~Fase 8 — CLI y toolchain básica~~: CLI, compilación, ejecución, `.axm`, ejemplos, tests y CI.

## Ecosistema inicial

- [x] ~~Fase 9 — Standard Library~~: built-ins iniciales y sus implementaciones semánticas y de runtime.
- [x] ~~Fase 10 — Package Manager~~: proyectos, dependencias locales, `vendor/`, lockfile y paquetes.
- [x] ~~Fase 11 — CLI como workflow de proyecto~~: `new`, `test`, `package` y estructura de proyectos.
- [x] ~~Fase 12 — AXIOM Studio~~: editor, diagnósticos, completion, ejecución, tests, debugger, profiler, Git, paquetes, terminal, documentación e IR.

## Núcleo y autonomía del lenguaje

- [x] ~~Fase 13 — AXIOM Core~~: consolidar el modelo de tipos, módulos/imports/namespaces y resolución de nombres; separar formalmente frontend, IR y backend; definir API interna del compilador e interfaces estables de tooling; integrar CLI, Studio, paquetes y compiler; establecer fundamentos para bootstrap, compatibilidad entre implementaciones y especificación formal del lenguaje.
- [x] ~~Fase 14 — AXIOM Engine~~: núcleo general-purpose para aplicaciones, escenas, entidades/componentes, recursos, eventos, simulación determinista, networking y fronteras backend-neutral de render, UI y audio.
- [x] ~~Fase 15 — Backend nativo y compilación real~~: backend AOT sobre el runtime nativo, targets explícitos, ejecutables standalone, cross-compilation mediante targets Rust y builds reproducibles.
- [x] ~~Fase 16 — Bootstrap de AXIOM~~: seed inicial del compilador escrito en AXIOM, modelo de tokens/AST, lowering a `AXIOM_ARTIFACT_V1` y ciclo reproducible AXIOM → artefacto nativo.
- [ ] **Fase 17 — Self-hosting completo**: frontend, semántica, IR y pipeline del compilador escritos en AXIOM; el compilador compila su propio código de forma reproducible.
  - [x] Lexer self-hosted.
  - [x] Parser self-hosted sobre una gramática amplia.
  - [x] Primera representación AST estructurada (`AXIOM_AST_V1`).
  - [x] Primera capa de análisis semántico sobre `AXIOM_AST_V1`.
  - [ ] Resolución de nombres, scopes y comprobación de expresiones en AXIOM.
  - [ ] Análisis semántico completo migrado a AXIOM.
  - [ ] Lowering AST→IR migrado a AXIOM.
  - [ ] Compiler driver y self-build reproducible.
- [ ] **Fase 18 — Independencia completa del ecosistema**: instalación y toolchain autónomos, runtime y package manager independientes, distribuciones multiplataforma y uso de `axiom build/run/test/package` sin Python como fundamento del lenguaje.

## Hito de independencia

El camino es deliberadamente progresivo:

Python/Rust → AXIOM compiler inicial → compiler progresivamente escrito en AXIOM → bootstrap reproducible → AXIOM compila AXIOM → toolchain autónomo.

Completar la Fase 17 significa alcanzar self-hosting. La Fase 18 lleva esa independencia desde el compilador al ecosistema completo del lenguaje.

## Relación con el roadmap maestro

`Documento sin título.txt` define el roadmap global de AXIOM SYSTEMS hasta la Fase 43. Este archivo cubre únicamente el sub-roadmap del lenguaje y su toolchain, y no marca como completadas las fases globales equivalentes por el mero hecho de completar una fase de este documento.

Este archivo se actualiza a medida que cada fase del lenguaje queda realmente implementada, probada y documentada.
