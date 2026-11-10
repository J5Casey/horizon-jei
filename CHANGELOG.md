# Changelog

## Unreleased: what the "+" lacks

- **Missing items in red**, as JEI's: with the pointer on a "+", the inputs the player lacks (inventory and the open
  crafting grid counted, a worn or renamed tool included) turn red, the button shows JEI's red disabled state, its
  tooltip says "Missing Items", and a click does nothing.
- **The server's word on the "+"**: with the pointer on a "+", the server tries the transfer and undoes it
  (Horizon's `native_transfer_check`, J5Casey braid/dev10) and the red follows its answer, so a Crafting
  Station's "+" counts the chests beside it, as the transfer itself does (and later any network storage a mod
  lends the grid). The answer is asked again as the inventory changes and once a second while the pointer
  stays; until it comes, a vanilla grid shows the client's own count and a mod's grid the plain "+".
- A worn Twine Rod still moved nothing into a Copper-Reel Rod's grid: Horizon's viewer transfer kept the recipe
  book's rule (`isUsableForCrafting`, no damaged, enchanted or renamed items). Fixed in Horizon on J5Casey
  braid/dev9 (`e643ca7`); checked in game with the Braid mods all pinned there.

## 0.2.2-braid.2: JEI 1.12's look

Jack's packs (Enigmatica 2 Expert, Divine Journey 2, Project Ozone 3) ran JEI 4.x
and HEI on 1.12.2; the screens now follow its source (`RecipesGui`,
`IngredientListOverlay`, `BookmarkOverlay`) number for number:

- **The recipe screen** is JEI's `RecipesGui`: 198 wide and as tall as the GUI
  less 68 (175 to 350), the vanilla darkened background over the screen under
  it, two header rows of small buttons with the category's name and "1/4" in
  white on dark bars, the tabs in pages of eight (a "1/2" row over them when
  there are more), the catalysts' column fused to its left edge, and the
  recipes spread down it with even room, each in its frame. Crafting recipes
  sit on the crafting table's own grid, arrow and big result slot, 2x2 ones in
  its corner, with JEI's shapeless icon; cooking recipes on the furnace's
  (smoker's, blast furnace's) with the flame and arrow burning, time and XP at
  the top right. Those backgrounds come from the player's own game files.
  The "+" is JEI's small button outside the frame at the bottom right, its
  icons tinted as JEI tints them; the slot under the pointer lights.
- **The item list** is JEI's overlay (Horizon's `NativePanelStyle::Classic`):
  flush with the screen's right edge, its "1/45" row of page buttons always
  there (they wrap around), the search bar at the foot of the screen with
  JEI's wrench beside it (red in cheat mode; it opens the settings page), red
  search text when nothing matches. The search bar sits under the screen's
  window (JEI's "center search bar"), or under the list: "Search bar" on the
  settings page. With the recipe screen open the list makes room for it, and
  the screen under it shows no tooltips.
- **Bookmarks** are JEI's left list: from the screen's left edge to the
  window, their own "1/1" page row, as many columns as fit, and the ribbon
  button in the bottom left corner that shows or hides them. They stay over
  the recipe screen. The file now holds up to 1024.
- Needs Horizon's `NativePanelStyle::Classic`: this build pins J5Casey/horizon
  `braid/panel-classic` (main `a3bf653` and that change).

## 0.2.2-braid.1: Braid's build on horizon-rust/horizon main

The unreleased recipe screen and "+" below, plus Horizon's recipe viewer API
(horizon-rust/horizon#13070, merged but not stamped with an API number yet, so
this build pins main `a3bf653`; it pinned J5Casey/horizon braid/dev4 to dev6
before the PR merged):

- **Other mods' lookups.** `native_recipe_lookup`: a quest book's item asks
  for its recipes (click), its uses (right click) or a key over it (R, U, and A
  bookmarks it); the recipe screen opens as from JEI's list.
- **Mods' categories.** `native_collect_recipe_categories`, asked again when
  `native_recipe_categories_version` moves: Void Fishing's loot pages and the
  like show as tabs with chances.

Checked on main `a3bf653` with Braid's mod set (2026-11-09): R and U over a stack, "+" and Shift "+" in the
crafting table and a Crafting Station, and the Chisel tab. Two quirks seen then, both outside JEI:

- The stick's planks across the top row of a crafting table came from the check, not the game: `horizon-ctl
  use` opens the table on the client only, so its 3x3 face showed the server's 2x2 inventory grid. A table
  opened as a player opens it (`interact`) gets the stick's "+" in the middle column, as vanilla.
- The Copper-Reel Rod was missing from the list, so the "+" clicked was another recipe's. Horizon numbered
  the recipe book from the client's own files while the server counts the world's data pack recipes (the
  `braid` pack), and the server's book entries then replaced other recipes; the pack's own recipes had no
  "+". Fixed in Horizon (issue horizon-rust/horizon#14435, PR #14437); checked in game on main `a3bf653` with that fix: the
  Copper-Reel Rod and Twine Rod "+" fill the inventory's grid and a crafting table and both craft.


## Unreleased: the recipe screen, and a "+" that needs no unlock

- **"+" works with no recipe-book unlock.** It moves a recipe in through
  `native_transfer_recipe`, Horizon's viewer transfer, so it no longer needs the
  recipe in the player's recipe book (JEI never did). It is on for a crafting
  recipe whenever a grid it fits is open: the inventory's 2x2, the crafting
  table, or a mod's crafting menu (a crafting station); grey elsewhere and for a
  3x3 recipe over the 2x2. After a "+" the recipe screen closes on the grid.
- **Data pack recipes.** The viewer asks the server for its recipe types
  (`datapack_sync`) and rebuilds when they arrive (`recipes_received`), so a
  data pack's own recipes are listed and their "+" names what the server has.
- **JEI's own recipe screen.** R, U, a click in the list or on a bookmark now
  open a window centred over the open screen, as JEI's `RecipesGui`: the tabs
  above it, the tab's catalysts in a column at its left (a mod's crafting block
  among the crafting table's), the tab's name and page arrows, as many recipes
  a page as fit, a "+" on each crafting recipe, tooltips with the R/U counts.
  Left click on a stack shows its recipes, right click its uses; R, U and A
  work over its stacks. Escape and Backspace go back a page and close it at the
  first; the wheel turns pages. The screen under it keeps its menu, grid and
  cursor stack, and "+" fills that screen. The item list beside it and the
  bookmarks still work. The 0.2 panel view stays on the settings page
  ("Recipes: in the item panel").
- **Categories of chances.** The core takes recipe categories whose outputs
  drop with a chance (a fishing loot table, a sieve): their own tab, catalysts,
  inputs, each output's chance (one decimal, "<0.1%"), count range and notes,
  long ones over several pages. The game feeds them once Horizon has a hook
  for mods to declare them.
- Needs hook points 411 (`native_gui_draw_item_with`, `native_gui_tooltip`).


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