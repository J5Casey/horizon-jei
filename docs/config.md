# Configuration

JEI keeps two kinds of files: the game's settings for the mod (one file per key) and its own folder in
the game's `config/`, shared by every mod as on Fabric and NeoForge.

## The game's settings, `.horizon-mod-data/jei/`

Written by `native_set_setting`, read with `native_setting`; they change from the mod's page in
Options → Mods → JEI, and from the chat commands.

| Key | Values | Default | What it does |
|---|---|---|---|
| `give` | `true` / `false` | `false` | "Click gives items": in creative a click on a panel item gives one, a right click a stack. In survival the click opens the recipes instead. `/jei give` toggles it. |
| `language` | `en` / `es` | `en` | The mod's own strings. Item names always come from the game in the player's language. |

The mod's keys (`recipe`, `uses`, `bookmark`) are the game's `mods.toml`-style bindings, set on the
same page: R, U and A by default.

## The mod's files, `config/jei/`

| File | What it is |
|---|---|
| `jei.cfg` | The mod's own options, `key=value` a line. |
| `bookmarks.txt` | The bar, one bookmark a line. |

Both are made with their defaults when missing, written when a setting changes, and read once when
the mod first needs them (a world opens, the settings page opens, or an item tooltip shows).
`/jei config` prints their paths.

### `jei.cfg`

| Key | Values | Default | What it does |
|---|---|---|---|
| `sort` | `name` / `id` | `name` | The item list's order: by the game's name (then id, then variant), or by id. The settings page's "Sort" button and `/jei sort name\|id` change it, and the open list is rewritten at once. |
| `hints` | `true` / `false` | `true` | The `R: Recipes (n)` and `U: Uses (n)` lines in item tooltips, wherever the game shows one. The bookmark mark (`★ Bookmarked`) shows either way. |
| `bookmarks` | `true` / `false` | `true` | Draws the bookmark bar over container screens. Turning it off keeps the bookmarks and still bookmarks the stack the A key points at. |

Unknown keys and unknown values are ignored: an option the file names badly keeps its default.

### `bookmarks.txt`

```
# JEI bookmarks: item<TAB>count<TAB>components
minecraft:diamond	1	
minecraft:potion	1	[potion="minecraft:strength"]
```

`components` is the game's syntax for a stack's data, the one `NativeStack.components` carries and
`/give` reads (empty for a plain stack). Two stacks are the same bookmark when their item and their
components match: a potion of strength and a potion of swiftness are two bookmarks.

Lines starting with `#` and blank lines are skipped; a line without a count is one. The file holds at
most 24 bookmarks, the bar's two columns of twelve; `/jei bookmarks clear` empties it.

## Commands

| Command | What it does |
|---|---|
| `/jei` or `/jei about` | The build and how many items and recipes the tables hold. |
| `/jei give` | Toggles "Click gives items". |
| `/jei sort` | Says the list's order. |
| `/jei sort name` / `/jei sort id` | Sets it. |
| `/jei bookmarks clear` | Empties the bookmark bar. |
| `/jei config` | The paths of `jei.cfg` and `bookmarks.txt`. |