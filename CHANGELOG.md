# Changelog

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