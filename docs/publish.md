# Publishing 0.1.0

What the registry needs (docs: <https://playhorizon.dev/docs/publish-a-mod>), and what is already
done.

## Done

| Step | State |
|---|---|
| Version in `horizon-mixin.toml` | `0.1.0` |
| Package | `jei-0.1.0.hnmod`, 15,835 bytes, SHA-256 `c5590e0d4618d435fc0c56abcc9cdabf253be694710068a3dd4643ac25466b5b` |
| Reproducible | `native.py package` on the committed tree gives those exact bytes |
| `cargo horizon-mod check` | passes: `native-mod, format native-1/1da354b/0d6b103, for 26.3`, signature verified |
| Checked against the Nightly players have | `native.py check --horizon latest` passes on `fbd7288`; the bundle pins `1da354b`, the revision `proof.py` and the game run were made on |
| Licence | MIT (`LICENSE`, `NOTICE` with JEI's copyright) |
| Signing key | minisign, id `FC3386F1F51ECF6A`; the public half is `horizon-jei.pub`, the private half lives outside the repository (`~/.config/horizon-jei/`) and is password protected |
| Signature | `jei-0.1.0.hnmod.minisig` (next to the package in `dist/`, and its text goes in the form) |

## Submit

Open <https://playhorizon.dev/mods/submit>, sign in with GitHub, and fill in exactly this:

| Field | Value |
|---|---|
| Kind | Native mod |
| File | `dist/jei-0.1.0.hnmod` |
| Id | `jei` |
| Version | `0.1.0` |
| For Minecraft | `26.3` |
| What changed | the 0.1.0 section of [`docs/submission-0.1.0.md`](submission-0.1.0.md) |
| Name | `JEI for Horizon` |
| Summary | `An item and recipe viewer for Horizon: every item beside container screens, with search, recipes and uses` |
| Description | the description of [`docs/submission-0.1.0.md`](submission-0.1.0.md) |
| Authors | `enderliker` |
| Licence | `MIT` |
| Source code | `https://github.com/enderliker/horizon-jei` |
| Icon | none yet (optional; a square `https://` PNG, at most 256 KiB) |
| Public key | `RWRqzx718YYz/OHRM+pAnFWlDv8Rrgar5C4NszQg3EKJWQZWNel5Sslf` |
| Signature | the text of `dist/jei-0.1.0.hnmod.minisig` |

"Submit for review" opens a pull request in `horizon-rust/horizon-mods`; a maintainer reads every line
of the bundle (native mods are compiled into the player's game) and answers there.

## Re-checking before you press submit

```sh
python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py check "$PWD" --horizon latest
python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py package "$PWD"
cp /home/ender/horizon-tools/horizon-mods/native/target/jei-0.1.0.hnmod dist/
cargo horizon-mod check dist/jei-0.1.0.hnmod \
  --key "$(sed -n 2p horizon-jei.pub)" --signature dist/jei-0.1.0.hnmod.minisig
```

If `--horizon latest` moved, update `compatibility.json` and `horizon-mixin.toml` to the new commit
(they must match), rebuild, re-run `cargo test` and `proof.py`, then sign again: the signature covers
the exact bytes.

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

A new version means a new `.hnmod`, a new signature with the same key and a new submission; every
version of a native mod is read again. After a Nightly, run `native.py check --horizon latest` and
re-pin as above (the CI job in `.github/workflows/nightly.yml` does it daily and opens an issue when
it breaks). The key cannot change through the form: a different public key is a pull request by hand.