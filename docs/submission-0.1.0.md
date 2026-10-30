# 0.1.0 — text for the submission form

Copy each block into <https://playhorizon.dev/mods/submit>. The rest of the fields are in
[`publish.md`](publish.md).

## Name

```
JEI for Horizon
```

## Summary

```
An item and recipe viewer for Horizon: every item beside container screens, with search, recipes and uses
```

## Description

```markdown
An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust reimplementation of
Minecraft: Java Edition. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems): JEI is the specification, the code here is
new Rust written against Horizon's native hook points.

**This is 0.1.0, the core viewer.** It is unofficial and not affiliated with mezz, Mojang, Microsoft
or the Horizon project.

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
- English and Spanish interface; item names always come from the game, in your language.
- `/jei` reports the table sizes.

### What it does not do yet

Bookmarks, click-on-slot (R and U only), tooltips, the creative inventory panel, the non-recipe
categories (anvil, grindstone, compost, fuel) and shift on `+`. Each one needs a hook that Horizon does
not expose yet; the matrix is in
[docs/jei-parity.md](https://github.com/enderliker/horizon-jei/blob/main/docs/jei-parity.md).

### What it needs

Horizon 26.3 with hook points version 2 (the current Nightly). Native mods are compiled into the game
and are **not sandboxed**: build only source you trust. It reads the game's tables, the panel, chat and
its own settings folder (`horizon-mods/.horizon-mod-data/jei/`), and writes nothing else.

### Install

1. Mods → JEI → Install.
2. Settings → Game → Native mods (experimental) → Build (the first build downloads Rust; 20–30
   minutes).
3. Play the **Native mods** instance.

### Licence

MIT. The recipe layout, the search grammar's behaviour and some translation strings come from JEI
(© mezz), attributed in `NOTICE`.
```

## What changed

```markdown
The first playable viewer, on Horizon's native hook points (version 2).

- The item panel beside container screens: every item the game knows, paged by the game, with a
  search box.
- JEI's search grammar: words, `"quoted phrases"`, `-exclusions`, `|` alternatives and the
  `@namespace` / `#tag` prefixes (`$tooltip` and `%creative tab` wait for their data).
- R shows an item's recipes, U its uses, over the panel or any slot of the container screen. A click on
  a panel item does the same.
- Recipe pages with one tab per station (crafting, furnace, blast furnace, smoker, campfire,
  stonecutter, smithing, brewing), the game's five layouts, cooking time and XP, and a Back button that
  walks the history.
- A recipe slot opens that stack's recipes.
- "+" places a recipe the recipe book holds into the open grid, greyed until the book has it (no shift
  yet: the panel's action event carries no modifiers).
- "Click gives items" (Options → Mods → JEI, or `/jei give`): in creative a click gives one, a right
  click a stack; in survival the same click opens the recipes.
- English and Spanish UI (`Language` on the mod's page). Item names always come from the game in the
  player's language.
- `/jei` reports the table sizes.
```

## Public key

```
RWRqzx718YYz/OHRM+pAnFWlDv8Rrgar5C4NszQg3EKJWQZWNel5Sslf
```

## Signature

The whole file as `minisign` writes it, including both comment lines:

```
untrusted comment: signature from minisign secret key
RURqzx718YYz/AZEFNcRh0lQFAgg8JsladNTOULcWqBnZFoFoApH0UvoQQkTk8wC/LfZpfJkrn7XRzSUOPUOfudUebBSHHdVYQ0=
trusted comment: jei 0.1.0 for Horizon 26.3
OqC3JuFFGuGNdxATsGIvM20nrld1ME2ks3tI2hfaMX2s5bCgwYjieUDk7FbDQD8zxd3KHqOYaUTz6T+njgUNCw==
```