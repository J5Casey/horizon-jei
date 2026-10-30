# JEI ↔ Horizon parity matrix

Reference: JEI 31.17.0 for Minecraft 26.3 (`JustEnoughItems/`, branch `26.3`, HEAD `9f285707e`).
Target: Horizon 26.3, native hook points API 7 (`crates/horizon-client/src/game/native_hooks.rs`),
item panel API 2 (`native_panel.rs`, `native_panel_recipes.rs`).

Status: **P** portable with today's hooks · **H** needs a new hook (named) · **X** not possible today (reason).
Phase: the phase in `tasks/plan.md` that delivers it.

## 1. Ingredient list and overlay

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Grid of every ingredient, paged | `Gui/overlay/IngredientListOverlay.java`, `overlay/ingredients/*` | `native_panel_items` (max 8192, 12 columns, pages) | P | 2 |
| Panel on the left or right, configurable width | `IngredientListOverlay`, `IIngredientGridConfig` | fixed right of the container, fixed width | H `panel config` | 3 |
| Scroll through the list with the wheel | `IngredientGrid`, `GridScrollMath` | game pages it; wheel goes to the hotbar | H `panel config` | 3 |
| Hide the list with a key | `IClientToggleState` | mod can send an empty list | P | 2 |
| Ingredient list in the creative inventory | `CreativeModeInventoryScreen` handler | panel only on `Screen::Inventory` | H `panel in creative` | 3 |
| Subtypes: potions, enchanted books, firework stars/rockets | `plugins/vanilla/ingredients/*Factory`, `subtypes/*` | `NativeStack.components` (vanilla item syntax) | P + data | 2 |
| Fluid ingredients (loader types) | `api/constants/VanillaTypes`, `plugins/vanilla/ingredients/fluid` | panel takes items only | H `fluid stack` | 4 |
| Custom ingredient types from mods | `IIngredientType`, `IModIngredientRegistration` | no registry for ingredient types | X (no plugin ABI; see §9) | — |
| Configurable sort stages (name, tag, mod, durability) | `ingredients/IngredientSorter*.java` | mod-side index | P | 2 |
| "Edit mode": hide ingredients from the list | `gui/config/EditModeConfig` | mod-side (own config file) | P | 3 |

## 2. Search

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Search box under the list | game-drawn (panel) | yes, `native_panel_event::Search` | P | 2 |
| Substring search on name and id | `gui/search/ElementSearch*` | mod-side index | P | 2 |
| `@mod` prefix | `ElementPrefixParser` | item ids carry the namespace; no mod names for content mods in `NativeItem` | P (namespace; mod display names need data) | 2 |
| `#tag` prefix | `ElementPrefixParser` | `NativeItem.tags` | P | 2 |
| `$tooltip` prefix | `ElementPrefixParser`, `getTooltipStrings` | no tooltip strings for an item list | H `tooltip text` | 3 |
| `%creative tab` prefix | `ElementPrefixParser` | no creative tab data | H `creative tabs` | 4 |
| Grammar: quotes, `-` exclusion, `|` alternatives, escape | `SearchTokenizer` | mod-side | P | 1 |
| Search modes (on / off / require prefix) | `common/config/SearchMode` | mod-side config | P | 2 |
| Completion popup and syntax highlighting | `SearchCompletionOverlay`, `SearchSyntaxHighlighter` | the search box belongs to the game; no draw hook | H `search box draw` | 3 |
| Ingredient aliases and advanced search registration | `api/search/ISearchStorageBuilderFactory` | none | X (plugin ABI) | — |

## 3. Recipes, categories and pages

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Crafting shaped / shapeless | `plugins/vanilla/crafting` | `CraftingShaped` / `CraftingShapeless` | P | 2 |
| Smelting, blasting, smoking, campfire | `plugins/vanilla/cooking` | 4 kinds + time/XP | P | 2 |
| Stonecutting | `plugins/vanilla/stonecutting` | `Stonecutting` | P | 2 |
| Smithing transform and trim | `plugins/vanilla/anvil/Smithing*` | `Smithing` | P | 2 |
| Brewing | `plugins/vanilla/brewing` | `Brewing` | P | 2 |
| Anvil | `plugins/vanilla/anvil/AnvilRecipeCategory` | not a recipe type; computed from inputs | H `info data` | 3 |
| Grindstone | `plugins/vanilla/grindstone` | not a recipe type; computed | H `info data` | 3 |
| Compostable | `plugins/vanilla/compostable` | internal game table, not datapack | H `info data` | 3 |
| Fuel per furnace type | `plugins/vanilla/cooking/fuel` | internal item property | H `info data` | 3 |
| Information pages (text for an item) | `api/registration/IRecipeRegistration#addIngredientInfo` | none | H `info data` | 3 |
| Arbitrary category layout, custom slot positions and drawables | `api/recipe/category/IRecipeCategory`, `IRecipeExtrasBuilder` | five fixed layouts: `Grid`, `Cooking`, `Single`, `Smithing`, `Brewing` | H `category layout` | 3 |
| Animated widgets (arrows, flames, progress) | `IDrawableAnimated`, `RecipeExtrasBuilder` | fixed arrows/flames drawn by the game | H `category layout` | 3 |
| More than 16 tabs / 256 recipes per page | panel limits | hard limits | H `panel limits` | 3 |
| Catalysts (which stations make a recipe) | `IRecipeCatalystRegistration` | tab icon only, no "used in" links | H `catalysts` | 3 |
| Hide/unhide recipes and categories at runtime | `IRecipeManager#hideRecipes` | none | H `recipe visibility` | 4 |
| Dynamic recipe providers (`IRecipeManagerPlugin`) | `api/recipe/advanced/*` | none | X (plugin ABI) | — |
| Recipe tree / focus navigation | `RecipeGuiLogic`, `IRecipeManager#createRecipeLookup` | mod-side history | P | 3 |

## 4. Interaction

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| R over an item shows recipes | `gui/input/*`, key mappings | `native_panel_event::Key` over panel **or screen slots** | P | 2 |
| U over an item shows uses | same | same | P | 2 |
| Click a panel item: recipes; right click: uses | `IngredientListOverlay` | `Clicked { index, right }` | P | 2 |
| Click a **screen** slot: recipes; right click: uses | `GuiContainerHandler` | only keys, no mouse events on screen slots | H `slot event` | 3 |
| Tooltip hint "R: recipes / U: uses" | `api/gui/handlers/IGuiContainerHandler` | game tooltip only | H `tooltip lines` | 3 |
| Ingredient ghost on the cursor (drag to a slot) | `gui/ghost/*`, `IGhostIngredientHandler` | none | H `cursor stack` | 3 |
| Hover an ingredient: highlight matching slots | `GuiContainerHandler` | none | H `slot highlight` | 4 |
| Back/forward through recipe pages | `RecipeGuiLogic` | mod-side history + `Back` event | P | 2 |
| Mouse wheel changes recipe page | `RecipesGui` scroll | game pages recipes | P | 2 |

## 5. Bookmarks

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Bookmark an ingredient (A key) | `gui/bookmarks/BookmarkManager` | none | H `bookmark overlay` | 3 |
| Bookmark bar beside the list, left or right | `BookmarkOverlay` | none | H `bookmark overlay` | 3 |
| Bookmark recipes, not only ingredients | `RecipeBookmark` | none | H `bookmark overlay` | 3 |
| Drag to reorder, right click to delete | `BookmarkList` | none | H `bookmark overlay` | 3 |
| Persist bookmarks across worlds | `BookmarkCodec` | mod-side file in `native_data_dir` | P once drawn | 3 |
| Bookmark tooltip features (count, ids) | `BookmarkTooltipFeature` | none | H `tooltip lines` | 4 |

## 6. Transfer and cheat mode

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| "+" fills the crafting grid from a known recipe | `api/recipe/transfer/*` | `native_place_recipe(book_id, all)` — inventory/crafting only, book recipes only | P | 2 |
| Shift-click fills as many as possible | same | `all = true` | P | 2 |
| Transfer to furnace, smoker, brewing stand, etc. | `RecipeTransferHandler`s | needs server-side container moves; remote servers impossible without protocol change | H `server transfer` (SP/LAN only) | 3 |
| Cheat mode: click gives an item in survival | `common/config/GiveMode` | `native_give` is creative-only | H `give` (server authority) | 4 |
| Delete items by dragging to a trash area | `DeleteItemHandler` | needs cursor + slot events | H `cursor stack` | 4 |

## 7. GUI and rendering

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Draw over a container screen | `RecipesGui`, `overlay/*` | screens draw after the HUD; the HUD is blurred behind them | H `screen frame` | 3 |
| Draw an item icon outside the panel | `DrawableIngredientRenderer` | only the panel draws items | H `item draw` | 3 |
| Nine-slice and animated sprites | `DrawableSprite`, `.png.mcmeta` | `HudBatch::sprite` on the GUI atlas | P | 3 |
| Custom tooltips with several lines | `ITooltipBuilder` | game item tooltip only | H `tooltip lines` | 3 |
| Own screen with tabs, page arrows, buttons | `RecipesGui` | panel recipe pages have them | P | 2 |
| Options/config button in the overlay | `overlay/elements/*` | settings page in Options > Mods | P | 2 |
| GUI scale and layout for odd sizes | `GuiProperties`, `ScreenHelper` | panel layout handles height | P | 2 |

## 8. Config, input and integration

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Rich config UI with sliders, lists, search | MezzConfig + `gui/config/*` | settings page = label buttons; mod can add `/jei config` | P (command) / H `settings widgets` | 3 |
| Client/server config split | `common/config/*` | mod settings are per instance | P | 3 |
| Rebindable keys | `IJeiKeyMappingBuilder` | `NativeMod.keys` + `native_mod_key` | P | 2 |
| Modifier keys (Ctrl/Shift/Alt) | `JeiKeyModifier` | raw key names only; modifiers arrive as their own keys | P (partial) | 3 |
| Recipe sync on multiplayer servers | `VanillaClientRecipeLoader`, packets | the game syncs recipes; `native_recipes` is what the client knows | P | 2 |
| GUI handlers for other mods' screens | `IGuiContainerHandler` | no plugin ABI; native mods can hook screens directly | X | — |
| Debug plugin with fake categories | `Debug/` | `#[cfg(test)]` + a dev-only category flag | P | 2 |

## 9. Plugin API (the Java mod's ecosystem)

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| `IModPlugin` discovery (Fabric entrypoint / NeoForge annotation) | `startup/*`, `load/PluginLoader` | none: mods are compiled into one crate, no dependencies between mods | X (no plugin loader) | — |
| Registration objects (`IRecipeCategoryRegistration`, …) | `api/registration/*` | game-owned data + JEI's own categories | X | — |
| `IJeiRuntime`, `IIngredientManager`, `IRecipeManager` for other mods | `api/runtime/*` | none | X | — |
| Data-driven custom categories for content mods | — | proposed: JSON in the mod's data folder, read by the viewer | H `custom categories` | 4 |
| Compile-time cooperation between native mods | horizon-mixins modules (`pub(crate)`) | possible only when both bundles are built together and one names the other's module | P (fragile; not advertised) | 4 |

## 10. Localisation, assets and performance

| JEI feature | Source | Horizon today | Status | Phase |
|---|---|---|---|---|
| Item names in the player's language | game | `NativeItem.name` | P | 2 |
| Mod UI strings in the player's language | `assets/jei/lang/*.json` (31 files, ~630 keys) | game language not exposed to mods; own table + `language` setting | P (own table) | 2 |
| JEI textures (54 sprites) | `assets/jei/textures/gui/sprites` | MIT; reuse with `NOTICE` where the panel draws them | P | 3 |
| Async ingredient list build and search index | `IngredientListElementFactory`, Lucene | mod-side; measure (32768 items worst case is the panel cap) | P + measure | 2 |
| Search under 16 ms per keystroke | Lucene index | mod-side substring index; debounce if needed | P + measure | 2 |

## What v0.1 covers

Sections 1 (except side/scroll/creative), 2 (except `$`/`%`/completion), 3 (the nine vanilla recipe
kinds), 4 (R/U, clicks, back/forward), 6 (crafting transfer, creative give), 7 (panel pages, settings),
8 (keys, config command, server sync) and 10 (own localisation, performance budgets).

## What v1.0 needs from Horizon (the upstream RFC, task 3.1)

1. `native_screen` — the open screen's kind and GUI rect.
2. `native_screen_frame` — draw above the container screen.
3. `native_screen_event` — hover/click on the screen's slots, with the slot index.
4. `native_item` — draw an item icon into a `HudBatch`.
5. `native_tooltip` — add lines to an item's tooltip.
6. `native_cursor` — read and set the carried stack (ghost ingredients).
7. `native_panel_config` — side, width, scroll, creative inventory.
8. `native_panel_limits` — more tabs/recipes; category layout declaration.
9. `native_info` — anvil/grindstone/compost/fuel tables and ingredient info pages.
10. `native_tooltip_text` — tooltip strings for `$` search.
11. `native_give` outside creative (server-authoritative).
12. `native_language` — the player's language for mod UI strings.

Each one is additive and can be a helper plus, where needed, a hook point; all fit Horizon's rule that
hook points are only ever added.