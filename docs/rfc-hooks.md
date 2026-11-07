# RFC: the hook points JEI still needs

Status: draft (7 Nov 2026), for the Horizon maintainers. Written against Nightly 2026-11-07,
`6560641`, hook points API 411.

0.2.0 reaches JEI's overlay with what exists today: `native_screen_render_post`, `native_tooltip`,
the screen mouse hooks, `native_item_stack`, `native_config_dir` and the panel's `Key` event. What
follows is what is still out of reach. Each item says what a viewer needs, the signature that would
give it in the shape of the hook points already there, and the smallest use that proves it.

All of them are additive — a helper plus, where a place is needed, a hook point — so they fit
Horizon's rule that hook points are only ever added.

## 1. The slot under the pointer, and its index

```rust
/// The open menu's slot under the pointer, as `Game::slots` numbers it.
/// API 412.
pub(super) fn native_screen_slot(&self, screen: &NativeScreenView<'_>) -> Option<usize>
```

**Use:** JEI opens an item's recipes on a click of any slot, highlights the slots that match the
ingredient under the pointer, and drags an ingredient into one. Today the panel's `Key` event says
*which stack* is under the pointer but not *which slot*; a click on a slot has no mod-visible form at
all (`native_screen_mouse_button_pressed_pre` carries no slot).

**Smallest use:** with it, `mouse_press.rs` calls `Event::Open { stack, uses }` on a left click over a
slot — six lines in the mod, no geometry table.

## 2. The stack on the cursor

```rust
/// The stack the player carries (`Menu.getCarried`), and a way to set it for
/// the click the game is about to take. API 412.
pub(super) fn native_carried(&mut self) -> Option<ItemStack>
pub(super) fn native_set_carried(&mut self, stack: Option<&ItemStack>)
```

**Use:** the ghost ingredient (drag an item from the list to a slot) and the trash area, JEI's two
most used transfers after "+". The mod can draw a ghost today; it cannot make the game carry it.

**Smallest use:** in `screen_mouse_released`, set the carried stack from the drag's; the game's own
click then places it.

## 3. Placing an ingredient into a station

```rust
/// Puts `stack` where `slot` of the open menu takes it, as a server-side move
/// of the recipe book's `place`, for any menu. Single player and a LAN host
/// only. API 412.
pub(super) fn native_place_stack(&mut self, slot: usize, stack: &ItemStack, all: bool) -> Result<(), String>
```

**Use:** "+" in the furnace, the smoker, the brewing stand, the smithing table and the stonecutter.
`native_place_recipe` today only places a recipe the book holds, and only in the inventory's or the
crafting table's grid.

**Smallest use:** the "+" on a cooking page moves one ingredient into the menu's input slot.

## 4. The items' own data tables

```rust
/// The open menu's furnace fuel value of the item, in ticks (0: not a fuel).
/// API 412.
pub(super) fn native_fuel(&self, item: &ItemStack, furnace: bool) -> u32
/// The item's compost chance, 0 for one that composts nothing.
pub(super) fn native_compost(&self, item: &ItemStack) -> f32
/// The anvil's and the grindstone's result for their two inputs.
pub(super) fn native_anvil(&self, inputs: [&ItemStack; 2]) -> Option<ItemStack>
pub(super) fn native_grindstone(&self, inputs: [&ItemStack; 2]) -> Option<ItemStack>
```

**Use:** JEI's information categories: fuel per furnace type, compostable, anvil and grindstone. The
tables are in the game and are not datapack data, so no mod can rebuild them without duplicating
game tables that change every release.

**Smallest use:** a "Fuel" category listing what burns in a furnace and for how long.

## 5. An item's name for a components string

```rust
/// `stack`'s display name, with its components as the game names it
/// (`Potion of Strength`, `Arrow of Poison`). API 412.
pub(super) fn native_stack_name(&self, stack: &ItemStack) -> Text
```

**Use:** JEI's subtypes are *named* ingredients: enchanted books, firework rockets and stars, banner
patterns. 0.2.0 expands the variants the recipe tables name (potions, tipped arrows) because those
stacks already reach the panel with their components; their tooltip still says the base item's name.
Content mods' variants have no recipe entry at all, so this is also what makes them reachable.

**Smallest use:** the list's rows for `minecraft:enchanted_book` with each enchantment, named, each
opening its uses.

## 6. The game's language, and tooltip strings

```rust
/// The player's language, as `options.txt` says it (`en_us`, `es_es`).
/// API 412.
pub(super) fn native_language(&self) -> String
/// The tooltip lines the game would show for `stack`, without drawing them.
pub(super) fn native_tooltip_text(&self, stack: &ItemStack) -> Vec<String>
```

**Use:** the mod's own strings in the player's language (`Lang` has en/es and is chosen in the mod's
settings today), and JEI's `$tooltip` search, which matches a word in an item's tooltip.

**Smallest use:** `$shears` finds the items whose tooltip says "shears".

## Smaller ones, in one line each

| Need | Signature | For |
|---|---|---|
| Panel on the left, and its width | `native_panel_config(&mut self, panel: &NativePanelConfig)` | JEI's list placement and scrolling |
| The panel in the creative inventory for any mod | relax `native_panel.rs:201`'s `"recipes"` gate | the creative list |
| The creative tabs | `native_creative_tabs(&self) -> Vec<NativeTab>` | `%` search, tab filters |
| More than 16 tabs, 256 recipes, and a layout for a category | `native_panel_limits`, `NativeRecipeView::layout` | mods with many categories |
| Give outside creative | `native_give` on a server that allows it | cheat mode on a server |
| Fluids and energy as ingredients | `NativeStack` for a fluid/energy ingredient | modded parity |
| A screen hook that runs before the screen's own drawing | `native_screen_render_pre` | overlays the screen must not draw over |