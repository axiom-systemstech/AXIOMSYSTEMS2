# AXIOM Language

## Primer corte de sintaxis

La primera versión del lenguaje mantiene una sintaxis pequeña y legible:

```axiom
fn main() {
    print("Hello AXIOM")
}
```

Los bucles `for` aceptan inicializador, condición y actualización. `continue`
ejecuta la actualización antes de la siguiente iteración y `break` termina el
bucle actual:

```axiom
fn main() {
    for (let i: Int = 0; i < 5; i = i + 1) {
        if i == 2 {
            continue
        }
        if i == 4 {
            break
        }
        print(i)
    }
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
- Los literales `Float` usan punto decimal, por ejemplo `1.5`.
- La sintaxis prioriza una curva de aprendizaje corta sin renunciar a
  compilación nativa y ejecución rápida.

## Estructuras

La sintaxis propuesta para estructuras mantiene los campos explícitos y el
acceso directo mediante punto:

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

Los nombres de campo deben ser únicos dentro de la estructura. La primera
implementación soportará campos con tipos base y acceso de lectura; la
reasignación de campos y los tipos compuestos se añadirán después.

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

Los tipos `Int` y `Float` admiten operaciones aritméticas y comparaciones entre
operandos del mismo tipo:

```axiom
fn main() {
    let total: Float = 1.5 + 2.5
    print(total / 2.0)
}
```

El operador `%` calcula el resto de una división entre enteros:

```axiom
fn main() {
    print(17 % 5)
}
```

Los operadores `&&` y `||` usan cortocircuito: el operando derecho solo se
evalúa cuando el izquierdo no determina el resultado.

## Módulos

Los proyectos pueden dividirse en módulos `.ax` bajo `src/`. El nombre del módulo corresponde a su ruta relativa, usando puntos como separadores.

Ejemplo de módulo:

    module math
    fn double(value: Int) -> Int {
        return value + value
    }

Un archivo puede importar otros módulos con una ruta cualificada:

    module main
    import math

    fn main() {
        print(double(21))
    }

El compilador resuelve el grafo de módulos desde el archivo de entrada, rechaza módulos inexistentes y detecta ciclos de importación. Las unidades resueltas se analizan semánticamente como un único programa antes de generar AXIOM IR.

## Compiler Core

`axiom.core.Compiler` es la interfaz estable entre el workflow de proyecto y las capas internas del compilador:

    sources
      ↓
    ModuleResolver
      ↓
    AST
      ↓
    Semantic Analysis
      ↓
    AXIOM IR

El CLI usa esta misma interfaz cuando `axiom build` o `axiom run` reciben un directorio de proyecto. Studio puede consumir las mismas capas sin duplicar la lógica del compilador.

El modelo estructural de tipos está definido en `axiom.types`, con tipos primitivos, tipos nominales y arrays. El modelo se mantiene independiente de la representación concreta del backend para facilitar futuros backends nativos y el bootstrap.

## Contrato de compatibilidad

Las implementaciones Python y Rust mantienen sus propios frontends/runtime durante esta etapa. Las nuevas decisiones de sintaxis y tipos deben quedar cubiertas por pruebas de compatibilidad antes de retirar cualquiera de las implementaciones. La independencia completa se reserva para las fases de bootstrap y self-hosting del roadmap.
