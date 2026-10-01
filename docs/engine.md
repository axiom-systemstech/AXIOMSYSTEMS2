# AXIOM Engine

AXIOM Engine es el núcleo de ejecución generalista de la Fase 14. Coordina aplicaciones y subsistemas sin acoplar AXIOM a una GPU, backend de audio, implementación física o transporte de red concreto.

## Arquitectura

```text
Application
    │
    ▼
  Engine ──────────────── EventBus
    │                     SceneManager
    │                     ResourceStore
    │                     NetworkBus
    │                     SimulationClock
    │
    ├── RenderQueue
    ├── UIQueue
    └── AudioQueue
```

El núcleo proporciona fronteras de servicio. Las implementaciones concretas se incorporarán en las fases especializadas de Graphics, Physics, Audio y UI.

## Escenas, entidades y simulación

`Scene` contiene entidades, stores de componentes y recursos locales. `SceneManager` controla la escena activa. Los componentes son datos; el comportamiento se ejecuta mediante sistemas registrados en `Engine`.

`SimulationClock` separa el delta de frame del paso fijo de simulación, permitiendo ejecutar sistemas a una frecuencia estable aunque la presentación tenga frames de distinta duración.

## Eventos y recursos

`EventBus` proporciona publicación y suscripción síncrona con orden determinista. `ResourceStore` gestiona recursos vivos y `AssetCatalog` mantiene el registro declarativo de assets sin imponer cómo se decodifican.

## Networking

`NetworkBus` define una frontera de mensajes con inbox/outbox. No prescribe sockets ni protocolo: esos detalles pertenecen al transporte concreto.

## Presentación

`RenderQueue`, `UIQueue` y `AudioQueue` convierten operaciones de alto nivel en comandos backend-neutral. Graphics, UI y Audio pueden consumir esas colas sin que Engine conozca APIs específicas de plataforma.

## Aplicaciones

`Application` encapsula el lifecycle del Engine y proporciona un punto de entrada estable para futuras aplicaciones, servidores, herramientas, juegos y simulaciones.

## Alcance

Esta fase establece el motor general-purpose. GPU, física, audio y UI concretos permanecen desacoplados y se implementarán en sus fases especializadas del roadmap maestro.
