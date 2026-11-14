# JEI for Horizon

An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust
reimplementation of Minecraft: Java Edition 26.3. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems) (JEI), rebuilt on
Horizon's native hook points.

Status: **0.2.1 "Overlay and bookmarks"**. On top of 0.1's panel viewer (every
item, JEI's search grammar, R/U over any stack, a tab per station, recipe
pages, "+", creative give, English and Spanish), the mod now draws over
container screens:

- **Item tooltips** say what makes an item and what uses it (`R: Recipes (12)`,
  `U: Uses (3)`) and mark the bookmarked ones, in the game's own tooltips,
  the panel's included.
- **Subtypes**: the potions, tipped arrows and other stacks the recipe tables
  name with components are items of their own in the list.
- **Bookmarks**: the **A** key bookmarks the stack under the pointer; a bar of
  two columns of twelve sits at the screen's top left corner, left click opens
  a stack's recipes, right click its uses, shift or the middle button takes the
  bookmark away, and dragging reorders it. The bar is kept in
  `config/jei/bookmarks.txt` and comes back with the next world.
- **Config**: `config/jei/jei.cfg` holds the list's order (`/jei sort name|id`)
  and the toggles on the mod's settings page (tooltip hints, the bar, where
  recipes show).
- **Recipe screen** (unreleased): R, U and clicks open JEI's own window over the
  open screen, with tabs, catalysts, pages and a "+" that fills the grid under
  it with no recipe-book unlock. Escape or Backspace goes back. The 0.2 panel
  view is a setting.

See [`docs/jei-parity.md`](docs/jei-parity.md) for the feature-by-feature state
against JEI.

Not there yet: click on a container slot (only R/U), the creative inventory
panel, the non-recipe categories (anvil, grindstone, compost, fuel), shift on
"+", recipe transfer into furnaces, and the `$tooltip` / `%creative tab`
searches. Some of those need new Horizon hooks; [`docs/roadmap.md`](docs/roadmap.md)
lists them.

## Install (arismoko/horizon)

This branch (`arismoko`) is a flat native mod on the arismoko/horizon mod API:
`horizon-mod.toml` pins the game build, the launcher compiles `src/` against
that build's prebuilt SDK into the game. Braid installs it with
`python3 -I tools/braid_mods.py --fork sync --build` (its `mods.toml` names
this branch). By hand, pack the folder's `horizon-mod.toml`, `Cargo.toml`,
`src/` and `assets/` into `jei.hmod`, put it in the game folder's
`horizon-native-mods/` and run the fork launcher's
`native-build --trust-native-code`.

The pin is a local developer base (J5Casey/horizon `braid/ari-base`:
arismoko main plus `RecipesClientHost::recipe_book_ids`, which the "+" needs,
arismoko/horizon issue #4249) until that is merged and published.

## For other mods: the category API

JEI is also a library. A mod adds recipe categories (its own tab of
inputs and chance outputs) and opens the viewer on a stack through
`jei::api`, guarded with `mod_api::if_installed!(host, jei, ..)`. See
[`docs/category-api.md`](docs/category-api.md).

## Develop

```sh
cd engine && cargo test          # the viewer's engine, no game
mkdir -p .horizon/sdk && ln -s <horizon checkout at the pin> .horizon/sdk/api
cargo clippy --all-targets -- -D warnings   # the whole mod, against the SDK
```

In game: Options > Mods > JEI (click gives items, language, tooltip hints,
the search bar's place), the keys under Controls, and `/jei`, `/jei sort`,
`/jei bookmarks clear`, `/jei config`.

## Layout

```
horizon-mod.toml   the manifest and the [native] pin
src/lib.rs         the hooks
src/glue.rs        the game's side: panel, overlay, input, tooltips, files
src/catalog.rs     the game's items and recipes as the viewer takes them
src/engine/        the viewer itself, no Horizon types (engine/ tests it)
src/engine/api.rs  the library API other mods call
assets/jei/        JEI 1.12's GUI sprites (MIT) and the lang file
```

## License

MIT. See `LICENSE` and `NOTICE` (attribution to JEI, © mezz).
Not affiliated with mezz, Mojang, Microsoft or the Horizon project.
