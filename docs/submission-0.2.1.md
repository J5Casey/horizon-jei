# 0.2.1 — text for the submission form

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

**This is 0.2.1.** It is unofficial and not affiliated with mezz, Mojang, Microsoft or the Horizon
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
World switch cache invalidation and overlay updates.

- Fix: Clear cached viewer across world changes even when JEI is disabled. Previously, `sync` returned early when `native_on(jei::ID)` was false before the unload branch, keeping the previous world's viewer in memory across world transitions.
- Session management: Encapsulate viewer lifecycle in `Session` in `runtime.rs`, ensuring the cache is unconditionally cleared upon leaving a world.
- Regression test: Cover the disable -> switch worlds -> re-enable lifecycle sequence.
- Everything 0.2.0 added (bookmarks, tooltips, subtypes, sort order, and config) is included.
```

## Public key

```
RWRqzx718YYz/OHRM+pAnFWlDv8Rrgar5C4NszQg3EKJWQZWNel5Sslf
```

## Signature

The whole file as `minisign` writes it, including both comment lines:

```
untrusted comment: signature from minisign secret key
RURqzx718YYz/IMTqxXLr0yxt8BI4Ot8qWMwFObx9H5UjLTNtGMPzIIKcY3ZxSvKiIjsHXRUfYKnU6rgwOf5AF0Oo5iS+gS6KwE=
trusted comment: jei 0.2.1 for Horizon 26.3
wv3EhNQzQNHc43Xn0SirRtN5QHxE9jtuEgbRgyMfwIFL7rFvpsZYQGVVsKjZ6NQElq+5smbXNCOgWIGDI1YaDA==
```
