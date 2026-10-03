"""Deterministic, backend-agnostic AXIOM general-purpose engine kernel."""

from __future__ import annotations

from collections import defaultdict, deque
from dataclasses import dataclass, field
from time import monotonic
from typing import Any, Callable, Generic, Iterator, TypeVar

T = TypeVar("T")
EventHandler = Callable[[Any], None]

@dataclass(frozen=True)
class Entity:
    id: int
    name: str = ""

@dataclass(frozen=True)
class EngineEvent:
    name: str
    payload: Any = None

class EventBus:
    """Synchronous event bus with deterministic subscription order."""
    def __init__(self) -> None:
        self._handlers: dict[str, list[EventHandler]] = defaultdict(list)
        self._queue: deque[EngineEvent] = deque()

    def subscribe(self, name: str, handler: EventHandler) -> None:
        if handler not in self._handlers[name]:
            self._handlers[name].append(handler)

    def unsubscribe(self, name: str, handler: EventHandler) -> None:
        handlers = self._handlers.get(name, [])
        if handler in handlers:
            handlers.remove(handler)

    def emit(self, name: str, payload: Any = None) -> None:
        self._queue.append(EngineEvent(name, payload))

    def dispatch(self) -> int:
        dispatched = 0
        while self._queue:
            event = self._queue.popleft()
            for handler in tuple(self._handlers.get(event.name, ())):
                handler(event.payload)
            dispatched += 1
        return dispatched

class ComponentStore(Generic[T]):
    """Typed-by-convention component storage indexed by entity id."""
    def __init__(self) -> None:
        self._values: dict[int, T] = {}

    def set(self, entity: Entity, value: T) -> None:
        self._values[entity.id] = value

    def get(self, entity: Entity) -> T | None:
        return self._values.get(entity.id)

    def remove(self, entity: Entity) -> T | None:
        return self._values.pop(entity.id, None)

    def items(self) -> Iterator[tuple[int, T]]:
        return iter(self._values.items())

    def __len__(self) -> int:
        return len(self._values)

class ResourceStore:
    """Lifecycle-aware storage for engine-wide resources."""
    def __init__(self) -> None:
        self._resources: dict[str, Any] = {}

    def put(self, name: str, resource: Any) -> None:
        self._resources[name] = resource

    def get(self, name: str, default: T | None = None) -> Any | T | None:
        return self._resources.get(name, default)

    def require(self, name: str) -> Any:
        if name not in self._resources:
            raise KeyError(f"resource not found: {name}")
        return self._resources[name]

    def remove(self, name: str) -> Any | None:
        return self._resources.pop(name, None)

    def names(self) -> tuple[str, ...]:
        return tuple(sorted(self._resources))

@dataclass
class Scene:
    name: str
    entities: dict[int, Entity] = field(default_factory=dict)
    components: dict[str, ComponentStore[Any]] = field(default_factory=dict)
    resources: ResourceStore = field(default_factory=ResourceStore)

    def create_entity(self, entity_id: int, name: str = "") -> Entity:
        if entity_id in self.entities:
            raise ValueError(f"entity id already exists: {entity_id}")
        entity = Entity(entity_id, name)
        self.entities[entity_id] = entity
        return entity

    def destroy_entity(self, entity: Entity) -> None:
        self.entities.pop(entity.id, None)
        for store in self.components.values():
            store.remove(entity)

    def store(self, component: str) -> ComponentStore[Any]:
        return self.components.setdefault(component, ComponentStore())

class SceneManager:
    def __init__(self) -> None:
        self._scenes: dict[str, Scene] = {}
        self._active: str | None = None

    def add(self, scene: Scene) -> None:
        if scene.name in self._scenes:
            raise ValueError(f"scene already exists: {scene.name}")
        self._scenes[scene.name] = scene
        if self._active is None:
            self._active = scene.name

    def get(self, name: str) -> Scene:
        try:
            return self._scenes[name]
        except KeyError as error:
            raise KeyError(f"scene not found: {name}") from error

    def activate(self, name: str) -> Scene:
        scene = self.get(name)
        self._active = name
        return scene

    @property
    def active(self) -> Scene | None:
        return self._scenes.get(self._active) if self._active else None

    def names(self) -> tuple[str, ...]:
        return tuple(sorted(self._scenes))

@dataclass(frozen=True)
class NetworkMessage:
    channel: str
    payload: bytes

class NetworkBus:
    """Transport-neutral messaging boundary for future network backends."""
    def __init__(self) -> None:
        self._inbox: deque[NetworkMessage] = deque()
        self._outbox: deque[NetworkMessage] = deque()

    def send(self, channel: str, payload: bytes) -> None:
        self._outbox.append(NetworkMessage(channel, bytes(payload)))

    def inject(self, message: NetworkMessage) -> None:
        self._inbox.append(message)

    def drain_outbox(self) -> tuple[NetworkMessage, ...]:
        result = tuple(self._outbox)
        self._outbox.clear()
        return result

    def drain_inbox(self) -> tuple[NetworkMessage, ...]:
        result = tuple(self._inbox)
        self._inbox.clear()
        return result

@dataclass(frozen=True)
class Frame:
    index: int
    delta: float
    elapsed: float

class SimulationClock:
    def __init__(self, fixed_step: float = 1 / 60) -> None:
        if fixed_step <= 0:
            raise ValueError("fixed_step must be positive")
        self.fixed_step = fixed_step
        self.elapsed = 0.0
        self.accumulator = 0.0
        self.frame_index = 0

    def advance(self, delta: float) -> tuple[Frame, int]:
        if delta < 0:
            raise ValueError("delta must be non-negative")
        self.elapsed += delta
        self.accumulator += delta
        steps = 0
        while self.accumulator >= self.fixed_step:
            self.accumulator -= self.fixed_step
            steps += 1
        frame = Frame(self.frame_index, delta, self.elapsed)
        self.frame_index += 1
        return frame, steps

@dataclass
class EngineContext:
    events: EventBus
    scenes: SceneManager
    resources: ResourceStore
    network: NetworkBus
    clock: SimulationClock

class Engine:
    """Application lifecycle and subsystem orchestration kernel."""
    def __init__(self, fixed_step: float = 1 / 60) -> None:
        self.context = EngineContext(EventBus(), SceneManager(), ResourceStore(), NetworkBus(), SimulationClock(fixed_step))
        self._systems: list[Callable[[EngineContext, float], None]] = []
        self._running = False
        self._last_time: float | None = None

    def add_system(self, system: Callable[[EngineContext, float], None]) -> None:
        if system in self._systems:
            return
        self._systems.append(system)

    def start(self) -> None:
        if self._running:
            raise RuntimeError("engine is already running")
        self._running = True
        self._last_time = monotonic()
        self.context.events.emit("engine.started")
        self.context.events.dispatch()

    def tick(self, delta: float | None = None) -> Frame:
        if not self._running:
            raise RuntimeError("engine is not running")
        if delta is None:
            now = monotonic()
            delta = 0.0 if self._last_time is None else now - self._last_time
            self._last_time = now
        frame, steps = self.context.clock.advance(delta)
        for _ in range(steps):
            for system in tuple(self._systems):
                system(self.context, self.context.clock.fixed_step)
        self.context.events.dispatch()
        return frame

    def stop(self) -> None:
        if not self._running:
            return
        self._running = False
        self.context.events.emit("engine.stopped")
        self.context.events.dispatch()
        self._last_time = None

    @property
    def running(self) -> bool:
        return self._running


@dataclass(frozen=True)
class RenderCommand:
    kind: str
    payload: tuple[Any, ...] = ()


@dataclass(frozen=True)
class UICommand:
    kind: str
    payload: tuple[Any, ...] = ()


@dataclass(frozen=True)
class AudioCommand:
    kind: str
    payload: tuple[Any, ...] = ()


class RenderQueue:
    def __init__(self) -> None:
        self._commands: list[RenderCommand] = []

    def submit(self, kind: str, *payload: Any) -> None:
        self._commands.append(RenderCommand(kind, tuple(payload)))

    def drain(self) -> tuple[RenderCommand, ...]:
        commands = tuple(self._commands)
        self._commands.clear()
        return commands


class UIQueue:
    def __init__(self) -> None:
        self._commands: list[UICommand] = []

    def submit(self, kind: str, *payload: Any) -> None:
        self._commands.append(UICommand(kind, tuple(payload)))

    def drain(self) -> tuple[UICommand, ...]:
        commands = tuple(self._commands)
        self._commands.clear()
        return commands


class AudioQueue:
    def __init__(self) -> None:
        self._commands: list[AudioCommand] = []

    def submit(self, kind: str, *payload: Any) -> None:
        self._commands.append(AudioCommand(kind, tuple(payload)))

    def drain(self) -> tuple[AudioCommand, ...]:
        commands = tuple(self._commands)
        self._commands.clear()
        return commands


@dataclass
class EngineServices:
    """Backend-neutral service boundaries owned by an Engine instance."""
    render: RenderQueue = field(default_factory=RenderQueue)
    ui: UIQueue = field(default_factory=UIQueue)
    audio: AudioQueue = field(default_factory=AudioQueue)


def create_default_engine(fixed_step: float = 1 / 60) -> Engine:
    engine = Engine(fixed_step)
    services = EngineServices()
    engine.context.resources.put("services", services)
    engine.context.resources.put("render", services.render)
    engine.context.resources.put("ui", services.ui)
    engine.context.resources.put("audio", services.audio)
    return engine


@dataclass(frozen=True)
class Asset:
    name: str
    kind: str
    source: str


class AssetCatalog:
    """Declarative asset registry; actual decoding belongs to backend phases."""
    def __init__(self) -> None:
        self._assets: dict[str, Asset] = {}

    def register(self, name: str, kind: str, source: str) -> Asset:
        if name in self._assets:
            raise ValueError(f"asset already registered: {name}")
        asset = Asset(name, kind, source)
        self._assets[name] = asset
        return asset

    def get(self, name: str) -> Asset:
        try:
            return self._assets[name]
        except KeyError as error:
            raise KeyError(f"asset not found: {name}") from error

    def names(self) -> tuple[str, ...]:
        return tuple(sorted(self._assets))


class Application:
    """Application-level lifecycle built on the engine kernel."""
    def __init__(self, engine: Engine | None = None) -> None:
        self.engine = engine or create_default_engine()
        self._configured = False

    def configure(self) -> None:
        self._configured = True

    def run_frame(self, delta: float) -> Frame:
        if not self._configured:
            self.configure()
        if not self.engine.running:
            self.engine.start()
        return self.engine.tick(delta)

    def shutdown(self) -> None:
        self.engine.stop()
