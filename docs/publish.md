# Publishing 0.1.0

What the registry needs (docs: <https://playhorizon.dev/docs/publish-a-mod>), and what is already
done.

## Done

| Step | State |
|---|---|
| Version in `horizon-mixin.toml` | `0.1.0` |
| Version in `compatibility.json` | n/a (it names the game and the commits) |
| Package | `jei-0.1.0.hnmod`, 15,835 bytes, SHA-256 `c5590e0d…466b5b` |
| `cargo horizon-mod check` | passes: `native-mod, format native-1/1da354b/0d6b103, for 26.3` |
| Built and tested on the newest Nightly | `native.py check` on `1da354b` (run it again with `--horizon latest` before submitting) |
| Licence | MIT (`LICENSE`, `NOTICE` with JEI's copyright) |

## Before submitting

1. **Public source repository.** The registry requires a public repo under a licence it can read
   (MIT, Apache-2.0, GPL...). This folder has a git history; push it to GitHub/GitLab and put the URL
   in the submission form.
2. **Run the checks again, against the Nightly players have:**

   ```sh
   python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py check "$PWD" --horizon latest
   python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py package "$PWD"
   cargo horizon-mod check /home/ender/horizon-tools/horizon-mods/native/target/jei-0.1.0.hnmod
   ```

   If `--horizon latest` moved, update `compatibility.json` and `horizon-mixin.toml` to the new
   commit (they must match), rebuild, and re-run `cargo test` and `proof.py`.
3. **Sign it (recommended).** Without a key, anyone with access to the account could publish an
   update; with one, every later version must be signed:

   ```sh
   minisign -G -p horizon-jei.pub -s horizon-jei.key          # once; keep the .key private
   minisign -S -s horizon-jei.key -m .../jei-0.1.0.hnmod
   cargo horizon-mod check .../jei-0.1.0.hnmod \
     --key "RW…" --signature .../jei-0.1.0.hnmod.minisig
   ```

## Submit

Open <https://playhorizon.dev/mods/submit>, sign in with GitHub, and fill in:

| Field | Value |
|---|---|
| Kind | native mod |
| File | `jei-0.1.0.hnmod` (+ `.minisig` if signed) |
| id / version / game | `jei` / `0.1.0` / `26.3` |
| Name | JEI |
| Summary | An item and recipe viewer for Horizon: every item beside container screens, with search, recipes and uses |
| Description | What 0.1.0 does and does not do (see `CHANGELOG.md`); say that it is unofficial, that it needs hook points version 2, and that it is not sandboxed |
| Authors | you |
| Licence | MIT |
| Source | the public repo URL |
| Icon | optional, square PNG ≤ 256 KiB |

"Submit for review" opens a pull request in `horizon-rust/horizon-mods`; a maintainer reads every line
of the bundle (native mods are compiled into the player's game) and answers there.

## What the reviewer will ask

- **What it reads and writes.** Only the game's tables, its settings folder
  (`horizon-mods/.horizon-mod-data/jei/`), the panel and chat. No network, no other files.
- **What it hooks.** Only the hook points (`native_frame`, `native_panel_event`, `native_mods`,
  `native_settings`, `native_settings_click`, `native_command`); no game internals, no private fields.
- **What it does on a server.** Client-side only; the game syncs the recipes. `native_give` and
  `native_place_recipe` go through the game's own checks (creative, the recipe book).
- **What it does not do yet.** No bookmarks, no overlay, no click on container slots, no anvil/
  grindstone/compost/fuel pages, no shift on "+". `docs/jei-parity.md` has the full matrix.

## Updates

A new version means a new `.hnmod` and a new submission; every version of a native mod is read again.
After a Nightly, run `native.py check --horizon latest` and re-pin as above (the CI job in
`.github/workflows/nightly.yml` does it daily and opens an issue when it breaks).