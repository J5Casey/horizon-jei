# ADR 0001 — JEI on Horizon: reimplement the behaviour, build on the hook points

Status: accepted (30 Oct 2026) · Supersedes: none

## Context

JEI 31.17.0 for Minecraft 26.3 is ~117.5k lines of Java in five modules. It is tightly coupled to
Minecraft's classes (`ItemStack`, `SlotDisplay`, `Screen`, `GuiGraphicsExtractor`), has its own loader
adapters (Fabric/NeoForge) and an imperative plugin API discovered at startup.

Horizon 26.3 is a Rust reimplementation. Its mod story is different: a native mod is a
[horizon-mixins](https://github.com/horizon-rust/horizon-mixins) bundle compiled into the game, with
stable **hook points** (`crates/horizon-client/src/game/native_hooks.rs`) that are only ever added, and
the option of pinning a Nightly's commit to hook the game's own functions.

The only interactive surface the hook points offer above a container screen is the **item panel**
(`native_panel_items`, `native_panel_recipes`, `native_panel_event`) with five fixed recipe layouts and
the game's own item/recipe tables (`native_recipes`, `native_known_recipes`, `native_place_recipe`,
`native_give`).

## Decisions

1. **Reimplement, do not transliterate.** Rust code mirrors JEI's behaviour (search grammar, category
   model, R/U, history, bookmarks later), not its class structure. JEI is the specification; its pure
   tests are ported as the correctness anchor.
2. **Stable hook points first (`hooks: 2`), no pinned Nightly.** The panel is the drawing surface for
   v0.1. Pinning is reserved for a clearly separated shim if an overlay is needed before upstream
   accepts new hooks.
3. **The game owns data; the viewer owns presentation.** Recipes and ingredients come from Horizon.
   New categories (anvil, grindstone, compost, fuel) need Horizon data, not a JEI-side plugin ABI.
4. **No plugin loader.** Horizon has no dependencies between mods and no runtime code loading. A
   compile-time `pub(crate)` API between native mods is possible but fragile, so it is not advertised;
   content mods get data-driven categories instead.
5. **Thin helpers, pure core.** `runtime.rs` has no game types (`cargo test` without the game);
   `glue.rs` is the only adapter; each hook is a one-function helper.
6. **No access to `Game`'s private fields.** Helpers could read them (they run in a child module of
   `game`), but field names are not part of the hook contract. Missing data is requested upstream.
7. **Config in files + `/jei`.** Horizon's settings page only has label buttons; rich options live in
   `horizon-mods/.horizon-mod-data/jei/` and are edited with the command.
8. **Testing pyramid.** Ported unit tests → `native.py check` (compiles the whole game) → `proof.py`
   with `horizon-ctl --offscreen` (screenshots, widgets, inventory) → manual checklist.
9. **MIT compliance.** JEI-derived code, strings and sprites carry the original notice in `NOTICE`.

## Consequences

- v0.1 works on every Nightly and can be published early, but lacks bookmarks, overlays, tooltips,
  click-on-slot and the non-recipe categories.
- Full parity is gated on Horizon adding hooks (screen drawing, item drawing, slot events, cursor,
  tooltips, panel options, info tables). Those additions are proposed as an upstream RFC (task 3.1).
- The mod must keep its data assumptions visible: subtypes, compost/fuel tables and anvil maths are
  version-sensitive and get their own tests and update checklist.
- Performance budgets are mandatory because the panel is re-sent on every search change.