# JEI for Horizon

An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust
reimplementation of Minecraft: Java Edition 26.3. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems) (JEI), rebuilt on
Horizon's native hook points.

Status: **Phase 0 (skeleton)**. See [`../tasks/plan.md`](../tasks/plan.md) and
[`../tasks/todo.md`](../tasks/todo.md).

## Install (once published)

1. In the Horizon launcher: Mods → JEI → Install.
2. Settings → Game → Native mods (experimental) → Build. The launcher compiles
   the game with the mod; the first build downloads Rust and takes 20-30 minutes.
3. Play the **Native mods** instance.

## Develop

```sh
cargo test                       # the mod's own logic, no game

# From a clone of https://github.com/horizon-rust/horizon-mods:
python3 native/tools/native.py check /path/to/horizon-jei
python3 native/tools/native.py check /path/to/horizon-jei --horizon latest
python3 native/tools/native.py package /path/to/horizon-jei
cargo horizon-mod check native/target/jei-<version>.hnmod
```

In game: Options → Mods → JEI, and `/jei`.

## Layout

```
Cargo.toml                 the mod's tests (lib = mixins/jei/runtime.rs)
compatibility.json         game 26.3, hook points 2, the tested revision
mixins/jei/
  horizon-mixin.toml       the bundle: modules and HEAD hooks
  runtime.rs               the core (becomes crate::game::jei)
  list.rs settings.rs click.rs command.rs event.rs   one helper per hook
```

## License

MIT. See `LICENSE` and `NOTICE` (attribution to JEI, © mezz).
Not affiliated with mezz, Mojang, Microsoft or the Horizon project.