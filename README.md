# AXIOM SYSTEMS

AXIOM SYSTEMS es un ecosistema tecnológico construido por fases. El primer
objetivo ejecutable es crear una herramienta capaz de transformar programas
AXIOM en software útil, empezando por `Hello World`.

## Estado actual

La Fase 1 establece la identidad, las decisiones técnicas y la automatización
mínima del proyecto. El bootstrap actual usa Python 3.11 o posterior para
permitir iterar sin depender de un toolchain nativo en la primera etapa.

## Desarrollo

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install -e '.[dev]'
axiom --version
axiom doctor
python -m pytest
```

## Roadmap

El roadmap completo está en [Documento sin título.txt](Documento%20sin%20t%C3%ADtulo.txt).
La siguiente entrega funcional será el lexer de AXIOM, seguido por el parser,
el análisis semántico y el compilador incremental.

## Principios

- Construir la mínima pieza que habilite la siguiente.
- Mantener interfaces pequeñas y comprobables.
- Priorizar claridad, seguridad y builds reproducibles.
- Medir antes de mover componentes críticos a una implementación nativa.

## Licencia

AXIOM SYSTEMS se distribuye bajo la licencia [MIT](LICENSE).