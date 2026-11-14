# JEI's category API

JEI shows the vanilla recipe kinds itself. Any other mod's recipes (a sieve's
drops, a fishing loot table, chisel groups) reach it through JEI's library
API: JEI publishes plain functions from its crate root, and a mod that wants
a tab calls them. This replaces Horizon's old engine-side `recipe_categories`
hook, which arismoko/horizon retired. Nothing here needs an engine change.

The API lives in [`src/engine/api.rs`](../src/engine/api.rs) (re-exported as
`jei::api`). It has no Horizon types: every argument is JEI's own plain data,
so a mod's call keeps building across JEI and Horizon updates while these
signatures hold. Version 1 (`jei::api::API_VERSION`):

```rust
pub fn set_category(category: Category);   // add, or replace by id (keeps its tab place)
pub fn remove_category(id: &str);
pub fn categories() -> Vec<Category>;      // what is published now
pub fn show_recipes(stack: Stack);         // open JEI on what makes `stack` (R)
pub fn show_uses(stack: Stack);            // open JEI on what uses `stack` (U)

pub struct Category {
    pub id: String,             // namespaced by the publishing mod: "salvage:sifting"
    pub title: String,          // the tab's name
    pub subtitle: String,       // a line under the header, "" for none
    pub icon: Stack,            // the tab's icon
    pub catalysts: Vec<Stack>,  // the blocks or tools that do it (JEI's catalyst column)
    pub recipes: Vec<ChanceRecipe>,
}
pub struct ChanceRecipe {
    pub inputs: Vec<Vec<Stack>>, // each slot's alternatives (cycled one a second)
    pub outputs: Vec<Chance>,
    pub notes: Vec<String>,      // lines under the recipe
}
pub struct Chance {
    pub stack: Stack,
    pub chance: f32,             // per operation, 0 to 1 ("20.0%", "<0.1%", "100%")
    pub min: u32,                // the count range when it drops ("x1-3"; 1..1 shows nothing)
    pub max: u32,
    pub notes: Vec<String>,      // tooltip lines: conditions, tips
}
pub struct Stack {
    pub item: String,            // "minecraft:diamond"
    pub count: u32,
    pub components: String,      // "" (component variants are not drawn yet)
}
```

All of them derive `Clone`, `Debug`, `PartialEq` and `Default` (except
`Chance`), and `Stack::item("minecraft:stick")` is a stack of one.

## How JEI uses them

- A category is a tab after the stations' (crafting table, furnaces,
  campfire, stonecutter, smithing table), in the order categories were first
  published. Its recipes draw in JEI's chance layout: the inputs, an arrow,
  then each output with its chance and count range.
- R over a stack lists the category recipes whose outputs hold it, U the ones
  whose inputs or catalysts hold it, and the item tooltips' "R: Recipes (n)" and
  "U: Uses (n)" count them.
- The calls only record what they are given, behind a lock. JEI reads them
  on its next frame: a changed category set rebuilds the viewer's tabs and
  counts, a `show_*` opens the recipe screen (over the open screen, or on a
  screen of JEI's own when none is open). Publishing the same category again
  changes nothing and costs nothing on JEI's side.
- A category is per process. On a dedicated server nothing reads it; publish
  from a client hook so every client has it (single player shares one process,
  so either side works there).

## Calling it from another mod

`horizon-mod.toml`:

```toml
[dependencies]
jei = { version = "^0.3.0-braid.1", optional = true }
```

The requirement is checked against JEI's `horizon-mod.toml` `version` with Cargo's semver rules,
and that version is a pre-release (`0.3.0-braid.1`), which a plain `^0.3` does not match: name the
pre-release.

The launcher adds the path dependency and a `jei` Cargo feature itself; don't
write them into `Cargo.toml` (arismoko/horizon `docs/modding/src/concepts/dependencies.md`).
Then, in the mod, guard every call so it builds and runs without JEI:

```rust
fn recipes_received(&self, host: &mut dyn ClientHost, _e: &mut RecipesReceivedEvent<'_>) {
    mod_api::if_installed!(host, jei, {
        jei::api::set_category(jei::api::Category {
            id: "salvage:sifting".into(),
            title: "Sifting".into(),
            subtitle: String::new(),
            icon: jei::api::Stack::item("salvage:salvage_bench"),
            catalysts: vec![jei::api::Stack::item("salvage:salvage_bench")],
            recipes: sifting_tables(host)
                .map(|t| jei::api::ChanceRecipe {
                    inputs: vec![vec![jei::api::Stack::item(&t.input)]],
                    outputs: t
                        .drops
                        .iter()
                        .map(|d| jei::api::Chance {
                            stack: jei::api::Stack::item(&d.item),
                            chance: d.chance,
                            min: d.min,
                            max: d.max,
                            notes: Vec::new(),
                        })
                        .collect(),
                    notes: Vec::new(),
                })
                .collect(),
        });
    });
}
```

`recipes_received` fires after a join and after every `/reload`, which is
when data-pack driven tables change; a mod whose tables come from its own
config can publish once at `client_setup` (or the first `client_tick_post`
in a world) instead. For the quest book's "show recipes" button:

```rust
mod_api::if_installed!(host, jei, {
    jei::api::show_recipes(jei::api::Stack::item(&quest.item));
});
```

## Who adopts it

| Mod | Before (braid/dev12) | On this API (all four wired, braid/ari-base 86c2820370) |
|---|---|---|
| braid-salvage | `recipe_categories` hook: the bench's sifting drops | `set_category("salvage:salvaging", ..)` and `"salvage:hammering"` on the first server tick after a data load (the odds need a level) |
| braid-chisel | `recipe_categories`: one tab of chisel groups | `set_category("chisel:chiseling", ..)` from the server's group load, one recipe per group (inputs: the group's blocks; outputs: each at chance 1) |
| braid-void-fishing | `recipe_categories`: the void's loot table | Meditation, Matter, Dew Jar, Ember Crucible and the fishing odds pages, from `client_tick_post` whenever they change |
| braid-quests | `show_recipes(stack)` engine helper | `jei::api::show_recipes` / `show_uses` (book clicks, R and U over items) |

## Limits (version 1)

- No custom layouts: every category draws as inputs, arrow, chance outputs.
  A layout callback would need a drawing type in the API, which would tie it
  to Horizon's; ask first.
- No "+" for category recipes: transfer is the engine's, for the vanilla
  crafting grid and the menus that lend it sources.
- Stacks are item ids; `components` is kept but not drawn.
