# Publishing a release

What the registry needs (docs: <https://playhorizon.dev/docs/publish-a-mod>), and what is already
done for 0.2.1. An update is the same submit form with a new version: it adds a `[[versions]]` to
`mods/jei.toml`, the entry 0.1.0 created.

## Done for 0.2.1

| Step | State |
|---|---|
| Version in `horizon-mixin.toml` | `0.2.1` |
| Package | `jei-0.2.1.hnmod`, 31,495 bytes, SHA-256 `d0a840069ff9fb877021829584df72441928a9283feb27df30d244837bc7f2af` |
| Reproducible | `native.py package` run twice on the committed tree gives those exact bytes |
| `cargo horizon-mod check` | passes: `native-mod, format native-1/6560641/ e8cd0aa, for 26.3`, signature verified |
| Checked against the Nightly players have | `native.py check --horizon latest` passes (tested on `fdc87ae` and pinned `6560641`) |
| Hook points | needs version 313 or later (`compatibility.json`, `hooks`); 0.1.0 needed 2 |
| Licence | MIT (`LICENSE`, `NOTICE` with JEI's copyright) |
| Signing key | minisign, id `FC3386F1F51ECF6A`; the public half is `horizon-jei.pub`, the private half lives outside the repository (`~/.config/horizon-jei/` and a copy in the owner's Windows profile) and is password protected |
| Signature | `jei-0.2.1.hnmod.minisig`, trusted comment `jei 0.2.1 for Horizon 26.3` |

## Submit

Open <https://playhorizon.dev/mods/submit>, sign in with GitHub, and fill in exactly this:

| Field | Value |
|---|---|
| Kind | Native mod |
| File | `dist/jei-0.2.1.hnmod` |
| Id | `jei` |
| Version | `0.2.1` |
| For Minecraft | `26.3` |
| What changed | the "What changed" section of [`docs/submission-0.2.1.md`](submission-0.2.1.md) |
| Name | `JEI for Horizon` |
| Summary | `An item and recipe viewer for Horizon: every item beside container screens, with search, recipes, uses, tooltips and bookmarks` |
| Description | the description of [`docs/submission-0.2.1.md`](submission-0.2.1.md) |
| Authors | `enderliker` |
| Licence | `MIT` |
| Source code | `https://github.com/enderliker/horizon-jei` |
| Icon | none yet (optional; a square `https://` PNG, at most 256 KiB) |
| Public key | `RWRqzx718YYz/OHRM+pAnFWlDv8Rrgar5C4NszQg3EKJWQZWNel5Sslf` |
| Signature | the text of `dist/jei-0.2.1.hnmod.minisig` |

"Submit for review" opens a pull request in `horizon-rust/horizon-mods`; a maintainer reads every line
of the bundle (native mods are compiled into the player's game) and answers there. Every version of a
native mod is read again, even an update of the same project.

## Re-checking before you press submit

```sh
python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py check "$PWD" --horizon latest
python3 /home/ender/horizon-tools/horizon-mods/native/tools/native.py package "$PWD"
cp /home/ender/horizon-tools/horizon-mods/native/target/jei-0.2.1.hnmod dist/
minisign -S -s ~/.config/horizon-jei/horizon-jei.key \
  -m dist/jei-0.2.1.hnmod -t "jei 0.2.1 for Horizon 26.3"
cargo horizon-mod check dist/jei-0.2.1.hnmod \
  --key "$(sed -n 2p horizon-jei.pub)" --signature dist/jei-0.2.1.hnmod.minisig
```

`minisign` asks for the key's password on the terminal. The signature covers the exact bytes: if
`package` ever gives a different SHA-256, do not sign the old file, sign the new one and update the
tables above.

If `--horizon latest` moved, update `compatibility.json` and `horizon-mixin.toml` to the new commit
(they must match), rebuild, re-run `cargo test` and `proof.py`, then sign again.

## What the reviewer will ask

- **What it reads and writes.** The game's items and recipes, the panel and chat, the screens' render
  pass and tooltips. It writes only `config/jei/` (`jei.cfg` for the options, `bookmarks.txt` for the
  bar). No network, no other files.
- **What it hooks.** Only the hook points: `frame`, `screen_render_post`,
  `screen_mouse_button_pressed_pre`, `screen_mouse_button_released_pre`, `screen_mouse_dragged_pre`,
  `tooltip`, and the 0.1.0 targets (`native_mods`, `native_settings`, `native_settings_click`,
  `native_command`, `native_panel_event`); no game internals, no private fields.
- **What it does on a server.** Client-side only; the game syncs the recipes. `native_give` and
  `native_place_recipe` go through the game's own checks (creative, the recipe book). The overlay draws
  only over container screens, and a click on the bar never reaches the world.
- **What it does not do yet.** No click on container slots (R and U only), no creative inventory panel,
  no anvil/grindstone/compost/fuel pages, no shift on "+", no recipe transfer into furnaces, no
  `$tooltip` / `%creative tab` searches. `docs/jei-parity.md` has the full matrix.

## Updates

A new version means a new `.hnmod`, a new signature with the same key and a new submission. After a
Nightly, run `native.py check --horizon latest` and re-pin as above (the CI job in
`.github/workflows/nightly.yml` does it daily and opens an issue when it breaks). The key cannot change
through the form: a different public key is a pull request by hand, so keep the private key and its
password backed up (the passphrase of 0.1.0 and 0.2.0 is stored in the owner's password manager; the
key itself, in `~/.config/horizon-jei/` and in the owner's Windows profile).