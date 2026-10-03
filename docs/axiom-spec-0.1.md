# AXIOM 0.1 — Especificación conceptual consolidada

Estado: diseño conceptual cerrado para iniciar implementación.

## 1. Identidad

AXIOM es un lenguaje universal, general-purpose, compilable y orientado a expresar significado humano de forma natural y formal. La sintaxis visible es pequeña; el modelo semántico es profundo.

Principio: natural en la expresión, formal en el significado.

AXIOM no depende de IA, red, servidor o servicio externo para compilar, verificar, optimizar o explicar un programa.

## 2. Modelo semántico

INTENCIÓN → ENTIDADES → RELACIONES → RECURSOS → CAPACIDADES → EFECTOS → RESTRICCIONES → CONTRATOS → PLAN → EJECUCIÓN.

Entidades internas: VALUE, DATA, RESOURCE, CAPABILITY, KNOWLEDGE.

Relaciones: produce, consume, transforma, lee, escribe, depende, llama, comunica, crea, libera, contiene y mide.

## 3. Sintaxis canónica

El núcleo visible queda reducido a: nombre:, nombre = expresión, nombre(...), decidir, repetir, cuando, usar, ofrecer, necesita, puede, prefiere, restringir, demostrar y modo.

Los bloques usan indentación. Las llaves no son delimitadores normales.

Ejemplo:

    usuario:
        nombre: texto
        edad: entero

    alex: usuario
        nombre = "Alex"
        edad = 25

    mostrar(alex.nombre)

Las funciones no requieren fn/def/function:

    sumar(a, b):
        a + b

El resultado normal de un bloque es su última expresión. return deja de ser pieza fundamental.

## 4. Tipos

Un tipo es conocimiento formal sobre una entidad: significado, estructura, identidad, capacidades, relaciones, estados, restricciones, contratos, representación y conocimiento verificable.

Tipos base: booleano, entero, real, decimal, texto, carácter, bytes, tiempo, duración y unidad.

Científicos: racional, complejo, vector, matriz, tensor, intervalo, probabilidad y números algebraicos.

Unidades y dimensionalidad son parte del sistema semántico.

No existe null universal. Ausencia y fallo son posibilidades semánticas.

Generalización: relaciones + conformance + restricciones + inferencia. No se adopta <T>, traits o herencia como fundamento.

## 5. Módulos y paquetes

usar modulo expresa dependencia semántica. ofrecer define la interfaz pública.

Un módulo describe entidades, relaciones, capacidades, efectos, contratos, fallos y plataformas.

Jerarquía: PROYECTO → PAQUETE → MÓDULO → ENTIDADES.

La identidad lógica no depende de la ubicación física de los archivos.

## 6. Resource Flow

AXIOM no obliga al programador a modelar ownership, borrowing o lifetimes.

Pregunta: qué necesita existir, quién lo usa, qué operaciones se hacen, qué dependencias existen, cuándo termina el último uso y qué representación minimiza el coste.

CREACIÓN → USOS → DEPENDENCIAS → ÚLTIMO CONSUMIDOR → LIBERACIÓN/REUTILIZACIÓN.

Se aplica a memoria, archivos, sockets, GPU, NPU, QPU, dispositivos, procesos, threads, handles, conexiones, energía, tiempo e información.

El compilador puede elegir copia, alias, view, movimiento interno, almacenamiento local, compartido, GPU, etc. Las restricciones explícitas tienen prioridad.

## 7. Concurrencia

La concurrencia es una propiedad del plan, no una colección de async/await/thread como sintaxis fundamental.

AXIOM descubre independencia, paraleliza cuando es beneficioso, sincroniza cuando es necesario, detecta conflictos y carreras, y mantiene secuencialidad cuando la semántica lo exige.

Targets: CPU, SIMD, GPU, NPU, QPU, procesos, máquinas y clusters.

Eventos: cuando condición: acción. Periodicidad: cada 10 ms: medir(sensor).

El tiempo real añade deadlines, periodos, jitter, prioridad, bloqueo y determinismo.

## 8. Efectos y capacidades

Efecto = qué hace una operación. Capacidad = qué está autorizado a hacer. Recurso = sobre qué entidad opera.

Los efectos se infieren y propagan transitivamente.

Ejemplos: filesystem.read, filesystem.write, network.read, network.write, camera.read, gpu, clock.read, secure_randomness, hardware.raw_memory.

necesita expresa requisito. puede expresa permiso/posibilidad. prefiere expresa preferencia. restringir expresa obligación.

Una dependencia no obtiene automáticamente todos los privilegios que conoce. Principio de mínimo privilegio.

## 9. Information Flow

El flujo de información se analiza aparte del flujo de recursos.

Los datos pueden tener clasificación y política. Una capacidad para escribir red no implica permiso para enviar cualquier dato.

El compilador debe detectar flujos prohibidos y explicar su procedencia.

Toda capacidad relevante debe tener procedencia explicable: quién la necesita, por qué, qué operación la usa, qué módulo la introduce y qué efecto produce.

## 10. Contratos y verificación

demostrar introduce una propiedad que AXIOM debe analizar.

Estados de conocimiento: DEMOSTRADO, COMPROBADO, MEDIDO, SIMULADO, OBSERVADO, SUPUESTO, DESCONOCIDO, REFUTADO y NO ANALIZADO.

AXIOM nunca presenta una observación o test como demostración matemática.

Los contratos pueden cubrir entradas, salidas, invariantes, estados, recursos, capacidades, efectos, memoria, tiempo, energía, distribución, determinismo y seguridad.

Los contraejemplos deben mostrarse cuando sea posible.

## 11. Tolerancia a fallos

Los fallos son posibilidades normales del modelo, no una arquitectura excepcional.

Conceptos: retry, timeout, fallback, restart, isolate, degrade, replace, propagate, recover y cancel.

El compilador conoce si repetir una operación es seguro mediante contratos de idempotencia. Nunca se reintenta ciegamente una operación con efectos potencialmente no repetibles.

## 12. Tiempo y determinismo

El tiempo es un recurso/efecto observable. Se distinguen duración, instante, reloj, periodo, deadline, latencia, jitter y prioridad.

restringir expresa garantías duras. prefiere expresa objetivos blandos.

Ejemplo:

    restringir:
        deadline = 1 ms
        determinismo = obligatorio

## 13. Distribución

La distribución utiliza el mismo modelo de entidades, recursos, efectos y restricciones.

Recursos adicionales: nodos, red, latencia, serialización, consistencia y particiones.

Contratos: atomicidad, idempotencia, consistencia, recuperación y timeout.

RPC, colas, sockets y shared memory son implementaciones, no modelos lingüísticos independientes.

## 14. Metaprogramación y reflexión

El lenguaje tratará código, tipos y esquemas como entidades inspeccionables.

Capacidades: inspección de tipos y módulos, inspección de contratos, generación de código, transformación de AST, generación desde schemas, bindings y documentación.

La metaprogramación conserva el mismo modelo semántico y respeta capacidades, efectos y restricciones.

## 15. Interoperabilidad y ABI

AXIOM tendrá un ABI propio y estable. El ABI describe representación, layout, alineación, llamadas, estructuras, errores, recursos, capacidades y versiones.

FFI con C, C++, Rust, Python, .NET, WASM, APIs nativas y hardware.

WebAssembly/Component Model será un target de interoperabilidad, no el fundamento de AXIOM.

## 16. Evolución

El lenguaje tendrá versiones, compatibilidad de fuente, compatibilidad semántica, compatibilidad de paquetes, compatibilidad ABI, deprecaciones y migraciones verificables.

Herramienta conceptual: axiom migrar.

Una migración produce diff, explica cambios y recompila antes de aceptarse.

## 17. Ecosistema

Standard Library oficial y amplia: texto, colecciones, sistema, archivos, red, seguridad, bases de datos, formatos, UI, gráficos, audio/video, GPU, IA, matemática, ciencia, simulación, hardware, embedded, testing, observabilidad y tooling.

Toolchain: new, check, build, run, test, package, add, install, native-build, explain, format, lint y migrate.

Build identity: proyecto + dependencias + compilador + configuración + target = identidad de build.

Supply chain: origen, identidad, versión, hash, firma, dependencias, capacidades y reproducibilidad.

## 18. Modos

Los modos son opcionales, combinables y extensibles.

Ejemplos: sistemas, embebido, tiempo_real, científico, alto_rendimiento, seguro, crítico, distribuido, hardware, GPU, web, móvil y cuántico.

Un modo modifica prioridades, restricciones, garantías y estrategias de compilación. Nunca crea un lenguaje distinto.

## 19. Explicabilidad

axiom explicar debe poder responder qué hace el programa, qué recursos utiliza, qué capacidades necesita, qué efectos produce, qué paralelismo encontró, dónde coloca los datos, qué optimizaciones aplicó, qué propiedades demostró, qué no puede demostrar y por qué eligió CPU/GPU/NPU/QPU.

Nunca depende de un servicio de IA.

## 20. UI, ciencia, hardware y futuro

El mismo lenguaje debe poder describir aplicaciones, servidores, sistemas, robots, drones, aeronaves, máquinas industriales, dispositivos, simulaciones, GPU/NPU/QPU y sistemas distribuidos, además de servir como base futura para AXIOM OS.

La abstracción ayuda pero nunca bloquea el acceso de bajo nivel.

## 21. Auditoría final de identidad

AXIOM no será una variación superficial de Python, Rust, C++, Go o Haskell.

No serán pilares: fn/def/function; class/struct como sistemas separados; ownership/borrow/lifetime; async/await; try/catch como arquitectura de fallos; import/include como copia textual; if/for/match como taxonomía fundamental; unsafe como mundo separado.

La identidad nace de: ENTIDAD + RELACIÓN + INTENCIÓN + RECURSO + CAPACIDAD + EFECTO + RESTRICCIÓN + CONOCIMIENTO + PLAN.

## 22. Especificación de referencia

La implementación debe tratar este documento como arquitectura conceptual. Si una implementación actual contradice esta especificación, se migra progresivamente; no se modifica la arquitectura para preservar accidentalmente el prototipo histórico.

La sintaxis concreta se estabilizará mediante pruebas ejecutables y una gramática formal antes de declarar AXIOM 0.1 estable.
