# AXIOM Language

## Primer corte de sintaxis

La primera versión del lenguaje mantiene una sintaxis pequeña y legible:

```axiom
fn main() {
    print("Hello AXIOM")
}
```

Este corte reconoce funciones, identificadores, cadenas, paréntesis, llaves y
el punto y coma opcional. La gramática crecerá junto con el parser y cada
decisión estable se documentará aquí.

El bootstrap en Python es temporal: sirve para validar rápidamente el diseño.
El lenguaje AXIOM no queda ligado a Python y su compilador podrá tener un
backend nativo cuando las mediciones y la estabilidad del lenguaje lo
justifiquen.

## Decisiones iniciales

- Los archivos fuente usan la extensión `.ax`.
- Las posiciones de los tokens son base 1 para línea y columna.
- Los errores léxicos incluyen el carácter y su posición.
- Las palabras reservadas se distinguen de los identificadores durante el
  lexing.
- La sintaxis prioriza una curva de aprendizaje corta sin renunciar a
  compilación nativa y ejecución rápida.

## Estructuras de control

La rama condicional ya admite encadenamientos de `else if`:

```axiom
fn main() {
    let value: Int = 2
    if value == 1 {
        print("one")
    } else if value == 2 {
        print("two")
    } else {
        print("other")
    }
}
```