# Architecture

The mod is one horizon-mixins bundle, `mixins/jei/`, compiled into the game as two modules of
`crate::game`:

```
runtime.rs  -> crate::game::jei        the viewer: pure Rust, no game types, `cargo test`
glue.rs     -> crate::game::jei_glue   the edges: game tables in, panel/give/place out,
                                       the overlay's state (the bar, the config)
screen_post.rs tooltip.rs mouse_press.rs mouse_release.rs mouse_drag.rs
                                       the overlay: what the game's screens give it
list.rs settings.rs click.rs command.rs frame.rs event.rs   one helper per hook
```

## Why this split

- `runtime.rs` is also the crate's library (`Cargo.toml`, `[lib]`), so all of the viewer's logic is
  tested with `cargo test`, with no game and no display.
- `glue.rs` is the only file that names Horizon's types (`NativeRecipes`, `NativeStack`,
  `NativePanelEvent`, ...). Everything that changes when Horizon changes lives there.
- Helpers stay one function each: read the hook's arguments, call `jei_glue::sync` or `::event`,
  write nothing else.

## Hook points used (version 313)

| Hook | Helper | What it does |
|---|---|---|
| `native_frame` | `frame.rs` | Builds the viewer once per world; clears it when the world closes; runs what a bookmark click left behind |
| `native_panel_event` | `event.rs` | Search, click, R/U, the bookmark key, tab, back, recipe-slot click, action button |
| `native_mods` | `list.rs` | The row in Options > Mods, with the R, U and A keys |
| `native_settings` / `_click` | `settings.rs`, `click.rs` | Give, language, order, tooltip hints, the bar |
| `native_command` | `command.rs` | `/jei` (counts), `/jei give`, `/jei sort`, `/jei bookmarks clear`, `/jei config` |
| `native_screen_render_post` | `screen_post.rs` | Draws the bookmark bar and its tooltip over a container screen |
| `native_tooltip` | `tooltip.rs` | Appends the recipe/use hints and the bookmark mark to any item's tooltip |
| `native_screen_mouse_button_pressed_pre` | `mouse_press.rs` | A click on a bookmark is the mod's (canceled), not the world's |
| `native_screen_mouse_button_released_pre` | `mouse_release.rs` | A click that never left its slot opens the stack's recipes |
| `native_screen_mouse_dragged_pre` | `mouse_drag.rs` | Dragging a bookmark over another slot moves it |

Helpers called from `glue.rs`: `native_recipes`, `native_known_recipes`, `native_panel_items`,
`native_panel_recipes`, `native_give`, `native_place_recipe`, `native_mod_key`, `native_setting`,
`native_set_setting`, `native_chat`, `native_on`, `native_player`, `native_config_dir`,
`native_fill`, `native_text`, `native_text_width`, `native_item_stack`.

The config hooks and the drawing ones take `&self`, so the helpers bound to them take `&Game`
(`value = "&*self"`); a click that has to change the panel is left in `glue.rs`'s pending list and run
by `frame.rs` the same frame, where the game is `&mut`.

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
| `Variant` | an item with components: a row of the list of its own |
| `Counts` | how many recipes make an item and how many use it, for the tooltips |
| `Bookmarks` | the bar's stacks, their file, and what adding/removing/moving one does |
| `Config` | the list's order, the tooltip hints, the bar |
| `Bar` | the bar's places: two columns at the screen's corner, hit tests included |

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
- The game forwards every key press over a stack to the panel's owner, in a panel item or any screen
  slot, and that is the bookmark key's only way in: it is not heard on a screen with a server title
  (a chest) or in creative, where the panel is the official `recipes` mod's. There, bookmarks can be
  used (the bar's clicks are the mod's) but not added.
- The world's tables are built once per world. A content mod that changes recipes at run time would
  need a reload hook; content mods load their recipes at start, so this is enough today.
- The bar's place is the screen's top left corner, the same on every screen: the panel's own layout
  is the game's, and the hooks do not say where it put its grid. On a very narrow screen the widest
  container windows (the merchant's) reach that corner.
- Components are the game's syntax (`[potion="minecraft:strength"]`), read back with the same
  `command_components` the panel uses, so a bookmark with components draws and stores its own stack.
  An item's name for a components string is not exposed, so the tooltip of a variant says its base
  item's name.

## Tests

1. `cargo test`: tokenizer (14 cases from JEI's `SearchTokenizerTest`), search prefixes, the viewer
   (list, search, tabs, uses, history, give, transfer, empty), and the en/es strings.
2. `native.py check`: the whole game compiles with the bundle, warnings are errors.
3. `proof.py`: an offscreen game driven with `horizon-ctl`; see `docs/spike.md`.
4. Manual: F1/F3, a LAN server, quitting and re-entering a world.

## Language

The game does not expose its language to mods. `Lang` (en/es) lives in the `language` setting and is
switched on the mod's page; the item names still come from the game in the player's language. Adding
a language means a variant in `Lang` and its strings; the rest of the UI reads them from there.

## Adding a new behaviour

1. Put the logic in `runtime.rs` with a unit test.
2. Convert its inputs/outputs in `glue.rs` only.
3. If it needs game data Horizon does not expose, add it to `docs/jei-parity.md` as `H` and to the
   hook list in `docs/roadmap.md` instead of reaching for a private `Game` field.