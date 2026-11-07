# Roadmap

The port is delivered in phases. Each one is playable on its own; the feature matrix in
[`jei-parity.md`](jei-parity.md) names the phase that delivers each JEI feature.

| Phase | What it delivers | State |
|---|---|---|
| 0 | Foundations: the bundle, `native.py` checks, one helper file per hook, the pure core with `cargo test` | done |
| 1 | The pure core: item index, search grammar, categories, ordering, navigation | done |
| 2 | The panel and **0.1.0 "JEI Core"**: item list beside container screens, R/U, recipe pages by station, history, "+", give, en/es | done |
| 3 | Overlay and **0.2.0 "Overlay and bookmarks"**: item tooltips, subtypes, the bookmark bar with its clicks and drags, the config file and the list's order | done |
| 4 | Overlay parity: click on container slots, the ghost ingredient, slot highlighting, the information categories (anvil, grindstone, compost, fuel), panel options | needs the hook points below |
| 5 | Ecosystem and 1.0: creative panel, fluids, recipe visibility, full configuration, more languages | needs the hook points below |

0.1.0 used only the hook points of API 2. 0.2.0 uses the screen hooks (API 78-93), the item tooltip
(API 12), the config folder (API 26) and `native_item_stack` (API 313), so it builds on any Nightly
whose hook points are version 313 or later.

## Hook points the later phases need

Each one is additive (Horizon only ever adds hook points), and each is a small change next to the
existing `native_*` functions in `crates/horizon-client/src/game/`.

| What JEI needs | Today | What it unlocks |
|---|---|---|
| Click and hover on container slots, with the slot's index | R/U only, on the stack; the mouse hooks give no slot | click-to-see, slot highlighting |
| The stack on the cursor, and placing a ghost | no | visual transfer, drag-to-slot |
| Place an ingredient for any recipe | only the recipe book, and only in the inventory or a crafting table | "+" in the furnace, the smithing table, the stonecutter |
| Give items outside creative | no | the cheat mode on a server that allows it |
| The panel in the creative inventory | the official `recipes` mod only | creative parity |
| Panel position, size and scrolling | fixed at the right, paged | layout parity |
| More than 16 tabs, more than 256 recipes, custom layouts | no | mods with many categories |
| The anvil, grindstone, compost and fuel tables, and ingredient info pages | internal to the game | the information categories |
| Tooltip text of an item, for the `$` search | no | `$` search, richer tooltips |
| Creative tab data, for the `%` search | no | `%` search, tab filters |
| Fluids and energy as ingredients | items only | modded parity |
| UI text in the game's language | item names only | full i18n (0.2.0 keeps the language in the mod's settings instead) |

`docs/spike.md` records what the panel's API does today and the workarounds 0.1.0 uses.
