# Changelog

## 0.2.1 — World switch cache fix

A patch release fixing viewer cache persistence across world transitions when JEI is disabled:

- **Fix:** Clear cached viewer on world unload even while JEI is disabled. Previously, `sync` returned early when `native_on(jei::ID)` was false before the unload branch, leaving the old world's viewer in memory across world switches.
- **Session management:** Encapsulate viewer lifecycle into `Session` in `runtime.rs`, ensuring the cache is unconditionally cleared upon leaving a world.
- **Regression test:** Cover the disable -> switch worlds -> re-enable lifecycle sequence.

Verified with `cargo test` (44) and `native.py check` on Nightly `6560641`.

## 0.2.0 — Overlay and bookmarks

The overlay JEI is known for, on Horizon's screen hooks. It needs hook points
of version 313 or later (`native_item_stack`); it was tested on Nightly
2026-11-07, `6560641`.

- **Bookmarks.** The **A** key bookmarks the stack under the pointer, over a
  panel item or any slot of the container screen. The bar sits at the screen's
  top left corner, two columns of twelve: the left button opens a stack's
  recipes, the right one its uses, shift-click or the middle button takes the
  bookmark away, and dragging a slot moves it. Each slot shows the stack, and
  hovering it draws the item's name and its actions.
- **Kept between sessions.** `config/jei/bookmarks.txt` holds the bar and
  `config/jei/jei.cfg` the rest; `/jei config` says where they are.
- **Item tooltips** now say `R: Recipes (12)` and `U: Uses (3)`, and mark the
  bookmarked stack with a star. They go into every tooltip the game shows,
  the panel's included, and count exactly: a stack with components counts the
  recipes the tables name for those components.
- **Subtypes**: every `(item, components)` pair the recipe tables name is an
  item of its own in the list, so the potions and the tipped arrows are
  there like JEI's subtypes. Up to eight variants per item.
- **The list's order**: by name or by id, from the settings page or
  `/jei sort name|id`, kept in the config file.
- New commands and settings: `/jei sort`, `/jei bookmarks clear`, `/jei config`;
  the settings page's buttons now cover the order, the tooltip hints and the bar.
- Protections: a click on the bar is the mod's, never the world's, and the
  overlay draws only over container screens.

Verified with `cargo test` (43) and `native.py check` on Nightly `6560641`.

## 0.1.0 — JEI Core

The first playable viewer, on Horizon's native hook points (version 2). No
Nightly is pinned: it builds on any Horizon whose hook points are version 2 or
later.

- The item panel beside container screens: every item the game knows, paged by
  the game, with a search box.
- JEI's search grammar: words, `"quoted phrases"`, `-exclusions`, `|`
  alternatives and the `@namespace` / `#tag` prefixes (`$tooltip` and
  `%creative tab` wait for their data).
- R shows an item's recipes, U its uses, over the panel or any slot of the
  container screen. A click on a panel item does the same.
- Recipe pages with one tab per station (crafting, furnace, blast furnace,
  smoker, campfire, stonecutter, smithing, brewing), the game's five layouts,
  cooking time and XP, and a Back button that walks the history.
- A recipe slot opens that stack's recipes.
- "+" places a recipe the recipe book holds into the open grid, greyed until
  the book has it (no shift yet: the panel's action event carries no
  modifiers).
- "Click gives items" (Options > Mods > JEI, or `/jei give`): in creative a
  click gives one, a right click a stack; in survival the same click opens the
  recipes.
- English and Spanish UI (`Language` on the mod's page). Item names always come
  from the game in the player's language.
- `/jei` reports the table sizes.

Verified on Nightly `1da354b` with `cargo test` (31), `native.py check` and an
offscreen `proof.py` run: row and settings, search, tabs, transfer, history and
an empty search.