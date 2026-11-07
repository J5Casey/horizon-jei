# 0.2.0 — text for the submission form

Copy each block into <https://playhorizon.dev/mods/submit>. The rest of the fields are in
[`publish.md`](publish.md). This is an update of the `jei` entry that already exists, so Id and
public key must be the ones 0.1.0 was published with.

## Name

```
JEI for Horizon
```

## Summary

```
An item and recipe viewer for Horizon: every item beside container screens, with search, recipes, uses, tooltips and bookmarks
```

## Description

```markdown
An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust reimplementation of
Minecraft: Java Edition. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems): JEI is the specification, the code here is
new Rust written against Horizon's native hook points.

**This is 0.2.0.** It is unofficial and not affiliated with mezz, Mojang, Microsoft or the Horizon
project.

### What it does

- A panel beside container screens with every item the game knows, paged, and a search box.
- JEI's search grammar: words, `"quoted phrases"`, `-exclusions`, `|` alternatives, `@namespace` and
  `#tag`.
- **R** shows an item's recipes, **U** its uses, over the panel or any slot of the container screen; a
  click on a panel item does the same.
- Recipe pages with one tab per station — crafting, furnace, blast furnace, smoker, campfire,
  stonecutter, smithing, brewing — in the game's five layouts, with cooking time and XP. **Back** walks
  the history, and clicking a recipe slot opens that stack.
- **+** places a recipe the recipe book holds into the open grid.
- Clicks can give items (Options → Mods → JEI, or `/jei give`): in creative one per click, a stack on
  right click; in survival the same click opens the recipes instead.
- Every tooltip the game shows says what makes the stack and what uses it — `R: Recipes (12)`,
  `U: Uses (3)` — and marks the ones in your bar.
- Stacks that differ by their components are items of their own in the list, like JEI's subtypes: the
  potions, the tipped arrows and the rest of what the recipes name.
- **Bookmarks**: **A** bookmarks the stack under the pointer. The bar sits at the screen's top left
  corner in two columns of twelve; the left button opens a bookmark's recipes, the right one its uses,
  shift or the middle button takes it away, and dragging moves it. It comes back with the next world.
- English and Spanish interface; item names always come from the game, in your language.
- `/jei` reports the table sizes, `/jei sort name|id` the list's order, `/jei bookmarks clear` empties
  the bar, `/jei config` says where its files are.

### What it does not do yet

Clicking a container slot (R and U only), the creative inventory panel, the non-recipe categories
(anvil, grindstone, compost, fuel), shift on `+`, transfer into furnaces, and the `$tooltip` /
`%creative tab` searches. Most of those need a hook Horizon does not expose yet; the matrix is in
[docs/jei-parity.md](https://github.com/enderliker/horizon-jei/blob/main/docs/jei-parity.md).

### What it needs

Horizon 26.3 with hook points version 313 or later (the Nightly of 2026-11-07, `6560641`, is the one it
was tested on). Native mods are compiled into the game and are **not sandboxed**: build only source you
trust. It reads the game's items and recipes, the panel, chat and tooltips, and writes only its own
folder, `config/jei/` (`jei.cfg` for the options, `bookmarks.txt` for the bar). No network.

### Install

1. Mods → JEI → Update.
2. Settings → Game → Native mods (experimental) → Build (the first build downloads Rust; 20–30
   minutes; a build of 0.1.0 is rebuilt in a few).
3. Play the **Native mods** instance.

### Licence

MIT. The recipe layout, the search grammar's behaviour and some translation strings come from JEI
(© mezz), attributed in `NOTICE`.
```

## What changed

```markdown
The overlay, on Horizon's screen hooks (hook points 313 and later).

- Bookmarks: A bookmarks the stack under the pointer, the bar sits at the screen's top left corner in
  two columns of twelve, hovering a bookmark draws its name and its actions, the left button opens its
  recipes, the right one its uses, shift or the middle button takes it away, and dragging reorders it.
  It is kept in `config/jei/bookmarks.txt` and comes back with the next world.
- Item tooltips now say `R: Recipes (12)` and `U: Uses (3)` and mark the bookmarked stack. They go into
  every tooltip the game shows, the panel's included, and count exactly: a stack with components counts
  the recipes the tables name for those components.
- Subtypes: every `(item, components)` pair the recipe tables name is an item of its own in the list,
  so the potions and the tipped arrows are there like JEI's subtypes.
- The list's order, by name or by id, from the settings page or `/jei sort name|id`, kept in
  `config/jei/jei.cfg` along with the tooltip hints and the bar.
- New commands: `/jei sort`, `/jei bookmarks clear`, `/jei config`.
- Protections: a click on the bar is the mod's, never the world's, and the overlay draws only over
  container screens.

Everything 0.1.0 did is still there.
```

## Public key

```
RWRqzx718YYz/OHRM+pAnFWlDv8Rrgar5C4NszQg3EKJWQZWNel5Sslf
```

## Signature

The whole file as `minisign` writes it, including both comment lines:

```
untrusted comment: signature from minisign secret key
RURqzx718YYz/DUK9HrTZR+0SXVMxpzsfDslLng2bZpsTq6MgyLh6+XhjK+qr3+wsMLpV9BgduCCq9U1n9uk1Vvc+FhD4O0yiww=
trusted comment: jei 0.2.0 for Horizon 26.3
e5PH+uK5q4V40if1GBwJ4mUhWfbH+XdC64Gtx+BDhK3PHbbrHv7B7lrLkHkLQ8D33V7TkPGKbBKwH5QVAo9UDQ==
```
