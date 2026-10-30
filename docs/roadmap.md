# Roadmap

The port is delivered in phases. Each one is playable on its own; the feature matrix in
[`jei-parity.md`](jei-parity.md) names the phase that delivers each JEI feature.

| Phase | What it delivers | State |
|---|---|---|
| 0 | Foundations: the bundle, `native.py` checks, one helper file per hook, the pure core with `cargo test` | done |
| 1 | The pure core: item index, search grammar, categories, ordering, navigation | done |
| 2 | The panel and **0.1.0 "JEI Core"**: item list beside container screens, R/U, recipe pages by station, history, "+", give, en/es | done |
| 3 | Overlay parity: bookmarks, tooltips, click on slots, ghost ingredient, the information categories (anvil, grindstone, compost, fuel), panel options | blocked on the hook points below |
| 4 | Ecosystem and 1.0: creative panel, fluids, recipe visibility, full configuration, more languages | blocked on the hook points below |

Phase 3 is the one that needs Horizon to grow. Everything in 0.1.0 uses only the hook points of API 2,
so it builds on any Nightly that has them.

## Hook points phase 3 and 4 need

Each one is additive (Horizon only ever adds hook points), and each is a small change next to the
existing `native_*` functions in `crates/horizon-client/src/game/`.

| What JEI needs | Today | What it unlocks |
|---|---|---|
| Draw over a container screen | no: only the panel at the right | bookmarks, the overlay, the item on the cursor |
| Draw an item icon in a mod's own overlay | no: only inside the panel | ghost ingredients, custom layouts |
| Click and hover on container slots, with the slot's index | R/U only, on the stack | click-to-see, drag, overlay buttons |
| Tooltip lines of the mod's own | only the game's tooltip | R/U hints, item details |
| The stack on the cursor, and placing a ghost | no | visual transfer, drag |
| Place an ingredient for any recipe | only the recipe book, and only in the inventory or a crafting table | "+" in the furnace, the smithing table, the stonecutter |
| Give items outside creative | no | the cheat mode on a server that allows it |
| The panel in the creative inventory | no | creative parity |
| Panel position, size and scrolling | fixed at the right, paged | layout parity |
| More than 16 tabs, more than 256 recipes, custom layouts | no | mods with many categories |
| Fluids and energy as ingredients | items only | modded parity |
| UI text in the game's language | item names only | full i18n (0.1.0 picks the language in the mod's settings instead) |

`docs/spike.md` records what the panel's API does today and the workarounds 0.1.0 uses.