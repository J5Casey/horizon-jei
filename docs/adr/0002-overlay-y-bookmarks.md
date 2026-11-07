# ADR 0002 — The overlay: the mod's own drawing over the game's screens

Status: accepted (7 Nov 2026) · Supersedes: ADR 0001's decision 2 in part

## Context

ADR 0001 chose the panel as the only drawing surface and reserved a pinned Nightly for a separate
shim. Horizon's hook points have since grown from 7 to 411 (Nightly 2026-11-07, `6560641`), and the
overlay 0.1.0 was missing is now reachable without touching Horizon's source:

- `native_screen_render_post(batch, screen)` (API 78) draws after the game drew the screen, with
  `screen.id`, `screen.rect`, `screen.slots` and `screen.mouse`.
- `native_tooltip(stack, lines)` (API 12) adds lines to every item tooltip the game shows, the
  panel's included.
- `native_screen_mouse_button_pressed_pre` / `_released_pre` / `_dragged_pre` (API 80/81/82) hear
  the mouse over the screen and can cancel a press.
- `native_item_stack`, `native_fill`, `native_text`, `native_text_width` draw; `native_config_dir`
  (API 26) gives a shared `config/` folder.
- The panel's `Key { key, stack }` event already resolves the stack under the pointer, in a panel
  item **or any slot of the container screen**, so the bookmark key needs no slot geometry.

## Decisions

1. **Draw the overlay in `native_screen_render_post`, never by patching Horizon.** Everything 0.2.0
   does is a hook point of API 313 or less; nothing pins a commit in a way that breaks the "builds on
   any newer Nightly" rule. `compatibility.json` names the newest hook used.
2. **The overlay is a pure layout plus a thin edge.** `Bar`, the hit test, the tooltip lines and the
   bookmarks live in `runtime.rs` (unit-tested, no game types); `glue.rs` holds the state and reaches
   the game. One helper file per hook, as before.
3. **The hooks that only borrow the game defer.** `native_screen_render_post`, `native_tooltip` and
   the mouse hooks take `&self`, and opening the recipes needs `&mut`. A click that has to change the
   panel leaves an `Event` in the glue's pending list, and `native_frame` — the one hook per frame
   with `&mut` — runs it the same frame. No new state machine, no delay a player can see.
4. **The bar's place is the screen's top left corner.** The panel's grid belongs to the game and the
   hooks do not say where it is; the corner is the one place free on a container screen at every GUI
   scale. JEI puts its bar beside its own list; a fixed corner is the closest this API allows, and it
   is documented as a limitation.
5. **Bookmarks are stored in `config/jei/bookmarks.txt`, options in `config/jei/jei.cfg`**, in the
   game's shared `config/` folder, as on Fabric. A plain `key=value` file and one bookmark a line,
   which the pure layer parses and writes, keeps the crate dependency-free (no serde) and every rule
   unit-tested. ADR 0001's `native_data_dir` setting files stay for `give` and `language`, which the
   game's own settings page owns.
6. **Subtypes are data-driven.** An `(item, components)` pair the recipe tables already name — potions,
   tipped arrows — becomes a row of the list, capped at eight per item. The rest of JEI's subtypes
   (enchanted books, fireworks, banners) need an item's name for a components string, which no hook
   exposes: they are an upstream request, not an invented table in the mod.
7. **Only reliable data is shown.** The tooltip counts come from one pass over the recipe tables; no
   number is guessed from an internal game table (fuel, compost, anvil). Those stay in the roadmap.

## Consequences

- 0.2.0 needs hook points of version 313; an older Nightly builds the mod on the commit it pins
  instead, as the launcher does for any native mod.
- The bar can overlap a very wide container window (the merchant's) on a narrow screen, and it cannot
  be moved next to the list until the panel's geometry is exposed. Both are in `docs/jei-parity.md`.
- A click on the bar is the mod's: the press and its release are canceled, so no stack is taken and
  the world is not used underneath.
- The pending list is per frame and per thread; the single-player server thread never sees a screen
  hook, so no lock is needed.
- If Horizon later exposes the panel's layout and slot events (roadmap), the bar moves next to the
  list and the click-on-slot work lands on the same `Bar`/`Event` plumbing.
