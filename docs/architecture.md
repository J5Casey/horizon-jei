# Architecture

The mod is one horizon-mixins bundle, `mixins/jei/`, compiled into the game as two modules of
`crate::game`:

```
runtime.rs  -> crate::game::jei        the viewer: pure Rust, no game types, `cargo test`
glue.rs     -> crate::game::jei_glue   the edges: game tables in, panel/give/place out
list.rs settings.rs click.rs command.rs frame.rs event.rs   one helper per hook
```

## Why this split

- `runtime.rs` is also the crate's library (`Cargo.toml`, `[lib]`), so all of the viewer's logic is
  tested with `cargo test`, with no game and no display.
- `glue.rs` is the only file that names Horizon's types (`NativeRecipes`, `NativeStack`,
  `NativePanelEvent`, ...). Everything that changes when Horizon changes lives there.
- Helpers stay one function each: read the hook's arguments, call `jei_glue::sync` or `::event`,
  write nothing else.

## Hook points used (version 2)

| Hook | Helper | What it does |
|---|---|---|
| `native_frame` | `frame.rs` | Builds the viewer once per world; clears it when the world closes |
| `native_panel_event` | `event.rs` | Search, click, R/U, tab, back, recipe-slot click, action button |
| `native_mods` | `list.rs` | The row in Options > Mods, with the R and U keys |
| `native_settings` / `_click` | `settings.rs`, `click.rs` | "Click gives items" |
| `native_command` | `command.rs` | `/jei` (counts) and `/jei give` |

Helpers called from `glue.rs`: `native_recipes`, `native_known_recipes`, `native_panel_items`,
`native_panel_recipes`, `native_give`, `native_place_recipe`, `native_mod_key`, `native_setting`,
`native_set_setting`, `native_chat`, `native_on`, `native_player`.

## Data flow

```
world opens ─ native_frame ─► native_recipes() ─► glue::items/recipes ─► Viewer::new
                                                          │
                                                  Out::Items ─► native_panel_items
container screen opens ─ game draws the panel ─► player searches/clicks/keys
                                                          │
                                        NativePanelEvent ─► glue::event
                                                          │
                            Viewer::handle(Event, Context) ─► Vec<Out>
                                                          │
              panel items / recipe page / native_give / native_place_recipe / chat
```

`Viewer` owns the history: each open (R, U, click, recipe-slot click) pushes a page; `Back` pops it
and shows the item list when the history is empty.

## The viewer model

| Type | Meaning |
|---|---|
| `Item` | id, name, tags (what the search reads) |
| `Recipe` | kind, grid size, slots of `Ingredient`s, results, cooking time/XP, book id |
| `Ingredient` | index into the viewer's items, count, components |
| `Stack` | an item id, count and components: what the panel takes |
| `Kind` | the nine vanilla recipe kinds |
| `Station` | the eight stations; one tab each, in a fixed order, each with an icon item |
| `Layout` | the panel's five layouts (`Grid`, `Cooking`, `Single`, `Smithing`, `Brewing`) |
| `View` | a recipe as the panel draws it, including the note and the action button |
| `Event` | what the player did, in the viewer's words |
| `Out` | what the game is to do |
| `Context` | the keys, the give setting, creative, and the book's ids |

Mapping a recipe: `Kind -> Station` picks the tab, `Kind -> Layout` picks the drawing. Adding a
category means adding a `Kind` (from a game hook) and its mapping.

## Limits that shaped the design

- The panel takes at most 8192 items, 256 recipes per page, 16 tabs and 9 slots per recipe; the game
  pages the list and the recipes.
- **An empty item list clears the panel**, and with it the search box and future events. `glue.rs`
  sends one invisible slot (`minecraft:air`, count 0) when a search matches nothing.
- **Recipes with no ingredients** exist in `native_recipes` (special crafting recipes). They are
  dropped: a recipe without ingredients is not useful in a viewer.
- `native_place_recipe` only works for recipes the book holds and only in the inventory/crafting
  grid. `+` is shown greyed when the book doesn't hold the recipe, and the game refuses otherwise.
- Creative is not exposed; `native_give` is tried and, when it refuses, the same click opens the
  recipes instead (JEI without cheat mode).
- The world's tables are built once per world. A content mod that changes recipes at run time would
  need a reload hook; content mods load their recipes at start, so this is enough today.

## Tests

1. `cargo test`: tokenizer (14 cases from JEI's `SearchTokenizerTest`), search prefixes, the viewer
   (list, search, tabs, uses, history, give, transfer, empty).
2. `native.py check`: the whole game compiles with the bundle, warnings are errors.
3. `proof.py`: an offscreen game driven with `horizon-ctl`; see `docs/spike.md`.
4. Manual: F1/F3, a LAN server, quitting and re-entering a world.

## Adding a new behaviour

1. Put the logic in `runtime.rs` with a unit test.
2. Convert its inputs/outputs in `glue.rs` only.
3. If it needs game data Horizon does not expose, add it to `docs/jei-parity.md` as `H` and to the
   upstream RFC (`tasks/todo.md`, task 3.1) instead of reaching for a private `Game` field.