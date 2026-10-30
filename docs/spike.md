# Phase 0 spike: what the panel API does (30 Oct 2026)

Run on `nightly-2026-10-30-1da354b` (`1da354b`), Linux client built by `native.py` from WSL, with the
launcher's prepared resources for 26.3. Proof: `proof.py`, screenshots in
`<horizon-mods>/native/target/proof/jei/`.

Result: `{"proof": "jei", "ok": true, "filtered": 63}`.

## Confirmed

| Assumption | Evidence |
|---|---|
| `native_recipes()` gives items and recipes in a world | the panel listed 1658 items; Diamond had shapeless, smelting and stonecutting recipes |
| `native_panel_items` shows a list beside any container screen | screenshot `1-items.png` (crafting table, 6 columns at this GUI scale) |
| The game draws and pages the panel | page arrows and `n/m` text appear when the list overflows |
| `native_panel_event::Search` arrives with the box's text | typing `diamond` filtered the list to 63 items |
| `native_panel_event::Key` arrives with R over a panel item | `3-recipes.png` shows the recipe page for Diamond |
| `native_panel_recipes` draws tabs, layouts, notes and the action button | `Grid`, `Shapeless`, cooking note `10 s, 1 XP`, `+` buttons |
| `Back` returns to the items | proof step 4 |
| The five fixed layouts cover the nine vanilla recipe kinds | mapping in `glue.rs` |
| Station tabs, the `+` action button and Back work | proof steps 4-8 |
| `+` places a recipe the book holds | after `/recipe give @s *` and giving the input, the crafting grid received the Block of Diamond |
| A recipe slot opens that stack's recipes, and Back walks the history | proof step 6 |
| The mod row, its page and settings persistence work (`native_setting`/`native_set_setting`) | proof step 1, file `horizon-mods/.horizon-mod-data/jei/give` |

## Surprises and decisions

1. **The panel disappears when the list is empty.** `Panel::show_items` clears the native panel when
   `items.is_empty() && recipes.is_none()`, and with it the search box and every later event. A search
   that matches nothing therefore had to keep one invisible slot (`minecraft:air`, count 0). Recorded in
   `glue.rs`.
2. **`horizon-ctl type` takes the text verbatim in a screen.** `--replace` and `--field` are only parsed
   on the title screens; in game the whole rest is typed (`\b` deletes one character). The proof types
   `--replace diamond` into the box in its first version — fixed with `\b` per character.
3. **Recipes with no ingredients exist in `native_recipes`** (special crafting recipes) and show as a
   lone result. They are filtered out of recipe lists: a recipe without ingredients is not useful in a
   viewer.
4. **The panel is narrow at the default GUI scale** (6 columns, ~108 GUI px on 1280x720). Titles are
   ellipsised by the game. This is why panel width/placement is an upstream ask, not a v0.1 fix.
5. **llvmpipe**: the proof runs on the CPU Vulkan driver. Loading takes ~83 s, the first frame ~45 s;
   the game itself is usable for scripted checks, but this is not a performance measurement.
6. **Tutorial toast overlaps the panel's top-right** in a fresh world. Not a mod bug; proofs should
   dismiss it or wait it out.

## Still to check (Phase 2)

- Creative inventory: the panel does not show there (`browser_sync` requires `Screen::Inventory`).
- `native_give` and `native_place_recipe` from a real click: done for place (with the book unlocked);
  give is only covered by the unit test and the settings toggle so far.
- Searching while a recipe page is open; leaving the world with a page open.
- Server (LAN) recipes: `native_recipes` sends only what the client knows.
- Shift on the action button: the `Action` event carries no modifier, so `all` is always false
  (needs a hook).
- What the proof had to set up and a player does not: `/recipe give @s *` (unlock the book) and the
  input item in the inventory. The `+` is greyed until both are true, as JEI's is.

## Timings (this machine, WSL2, 8 cores, llvmpipe)

| Step | Time |
|---|---|
| First `native.py check` (download, horizon-mixins, all dependencies) | ~10 min |
| Incremental `native.py check` | 30 s |
| First `native.py proof` build (`fast`) | 7 min 48 s |
| Game start to the control socket (prepared resources) | ~2 min |
| `proof.py` itself | ~1 min |

Put `horizon-mods/native/target` on the big disk: the first build is ~1.5 GB.