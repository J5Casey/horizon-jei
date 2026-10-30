# JEI for Horizon

An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust
reimplementation of Minecraft: Java Edition 26.3. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems) (JEI), rebuilt on
Horizon's native hook points.

Status: **0.1.0 "JEI Core"**. The panel viewer lists every item, searches with
JEI's grammar (`"phrases"`, `-exclusions`, `|`, `@mod`, `#tag`), R/U over any
stack opens its recipes or uses with a tab per station, a recipe slot opens
that stack's page, Back walks the history, "+" places a recipe the book holds,
clicks give items in creative when the setting is on, and the UI speaks
English or Spanish. See [`../tasks/plan.md`](../tasks/plan.md),
[`../tasks/todo.md`](../tasks/todo.md) and [`docs/jei-parity.md`](docs/jei-parity.md).

Not there yet: bookmarks, click-on-slot (only R/U), tooltips, the creative
inventory panel, the non-recipe categories (anvil, grindstone, compost, fuel),
and shift on "+". Those need new Horizon hooks; the plan explains which.

## Install

Once it is in the registry: Mods → JEI → Install, then Settings → Game →
Native mods (experimental) → Build, and play the **Native mods** instance.

Until then, from this folder:

```sh
horizon-launcher native-build /path/to/horizon-jei/mixins/jei --trust-native-code
```

The first build downloads Rust and takes 20-30 minutes; later builds recompile
only the game. Native mods are not sandboxed: build only code you trust.

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