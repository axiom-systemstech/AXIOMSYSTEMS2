from axiom.engine import (
    Engine,
    NetworkMessage,
    Scene,
    SimulationClock,
    create_default_engine,
)


def test_engine_lifecycle_and_fixed_step():
    engine = create_default_engine(fixed_step=0.1)
    ticks = []
    engine.add_system(lambda context, delta: ticks.append(delta))
    engine.start()
    frame = engine.tick(0.25)
    assert frame.index == 0
    assert len(ticks) == 2
    engine.stop()
    assert engine.running is False


def test_scene_entities_and_components():
    scene = Scene("game")
    entity = scene.create_entity(1, "player")
    scene.store("transform").set(entity, {"x": 1, "y": 2})
    assert scene.store("transform").get(entity) == {"x": 1, "y": 2}
    scene.destroy_entity(entity)
    assert scene.entities == {}
    assert len(scene.store("transform")) == 0


def test_scene_manager_switches_active_scene():
    from axiom.engine import SceneManager

    manager = SceneManager()
    first = Scene("menu")
    second = Scene("game")
    manager.add(first)
    manager.add(second)
    assert manager.active is first
    assert manager.activate("game") is second


def test_event_bus_preserves_subscription_order():
    seen = []
    engine = Engine()
    engine.context.events.subscribe("input", lambda value: seen.append(("a", value)))
    engine.context.events.subscribe("input", lambda value: seen.append(("b", value)))
    engine.context.events.emit("input", 7)
    assert engine.context.events.dispatch() == 1
    assert seen == [("a", 7), ("b", 7)]


def test_network_bus_is_transport_neutral():
    bus = Engine().context.network
    bus.send("state", b"abc")
    assert bus.drain_outbox() == (NetworkMessage("state", b"abc"),)
    bus.inject(NetworkMessage("input", b"jump"))
    assert bus.drain_inbox()[0].channel == "input"


def test_engine_service_queues_are_backend_neutral():
    engine = create_default_engine()
    render = engine.context.resources.require("render")
    ui = engine.context.resources.require("ui")
    audio = engine.context.resources.require("audio")
    render.submit("sprite", "player")
    ui.submit("button", "play")
    audio.submit("play", "music")
    assert render.drain()[0].kind == "sprite"
    assert ui.drain()[0].kind == "button"
    assert audio.drain()[0].kind == "play"


def test_clock_rejects_invalid_values():
    try:
        SimulationClock(0)
    except ValueError as error:
        assert "positive" in str(error)
    else:
        raise AssertionError("expected ValueError")


def test_asset_catalog_is_deterministic():
    from axiom.engine import AssetCatalog

    catalog = AssetCatalog()
    catalog.register("player", "texture", "assets/player.png")
    catalog.register("theme", "audio", "assets/theme.ogg")
    assert catalog.names() == ("player", "theme")
    assert catalog.get("player").kind == "texture"


def test_application_owns_engine_lifecycle():
    from axiom.engine import Application

    app = Application()
    frame = app.run_frame(1 / 60)
    assert frame.index == 0
    assert app.engine.running is True
    app.shutdown()
    assert app.engine.running is False
