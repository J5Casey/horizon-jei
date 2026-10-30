# Development environment

Everything below was checked on 30 Oct 2026 on WSL2 (Ubuntu, Rust 1.99.1, Python 3.14). Paths are the
author's; adapt them to your machine.

## Toolchain

| Tool | Where |
|---|---|
| Rust / Cargo | `~/.cargo` (1.99.1, the version Horizon's own native mods use) |
| `horizon-mods` (the registry repo, for `native.py`) | `~/horizon-tools/horizon-mods` |
| `cargo-horizon-mod` | install once: `cargo install --git https://github.com/horizon-rust/horizon cargo-horizon-mod` |
| Horizon source for a check | downloaded by `native.py` into `horizon-mods/native/source/<commit>/` |
| Build output | `horizon-mods/native/target/` (keep it off `/tmp`: a first build is ~1.5 GB) |
| Horizon launcher MCP | `horizon-launcher mcp`: it starts the client under agent control (`opencode mcp list` shows `horizon connected`) |

Do not clone `horizon-mods` inside this repo: `native.py` keeps sources and a large `target/` next to it.

## Linux build dependencies (WSL)

Horizon's client links ALSA, udev, Wayland/X11 and xkbcommon:

```sh
sudo apt install -y build-essential pkg-config libasound2-dev libudev-dev \
  libxkbcommon-dev libwayland-dev libx11-dev libxcursor-dev libxi-dev libxrandr-dev
```

If another `-dev` package is ever missing on a machine without sudo, a local sysroot works too
(`apt-get download` + `dpkg-deb -x`, then point `PKG_CONFIG_PATH` at its `pkgconfig/`); it is not
needed on the author's machine. `tools/dev-check.sh` adds a local sysroot automatically when one exists.

## Installed Horizon

| What | Value |
|---|---|
| Selected version | `nightly-2026-10-30-1da354b` |
| Commit | `1da354b67a509812c4141b67de2affd6de65c342` |
| Prepared resources | `<launcher>/versions/nightly-2026-10-30-1da354b/.local/horizon-client/26.3` |
| Instance home | `%APPDATA%\horizon` (`/mnt/c/Users/<you>/AppData/Roaming/horizon` in WSL) |
| Display | WSLg, `DISPLAY=:0` — the `--offscreen` proofs work from the shell |

`compatibility.json` names the commit above; update it when the Nightly changes (the playbook is in
[`publish.md`](publish.md)).

## Commands

```sh
NATIVE=~/horizon-tools/horizon-mods/native/tools/native.py

# The mod's own logic, no game (fast)
cargo test

# Compile the whole game with the mod, at the pinned commit (5-15 min the first time)
python3 "$NATIVE" check "$PWD"

# ... and against the newest Nightly (run after each Nightly)
python3 "$NATIVE" check "$PWD" --horizon latest

# Build the game with the mod (fast profile) and print the binary
python3 "$NATIVE" build "$PWD"

# Drive the built game end to end (needs the prepared resources, see above)
python3 "$NATIVE" proof "$PWD" --resources "$HORIZON_26_3"

# Package the .hnmod, then verify it as the registry does
python3 "$NATIVE" package "$PWD"
cargo horizon-mod check ~/horizon-tools/horizon-mods/native/target/jei-<version>.hnmod \
  --key "$(sed -n 2p horizon-jei.pub)" --signature <file.minisig>
```

`tools/dev-check.sh` runs format, clippy, tests, the compile check and the package. Measured times for
each step on the author's machine are in `docs/spike.md` (first check ~10 min, incremental check 30 s,
first `proof` build 7 min 48 s, game start ~2 min).

## Notes

- `native.py` takes an absolute path to the bundle; it reads `compatibility.json` and
  `mixins/<id>/horizon-mixin.toml` from there. `package` writes into the `horizon-mods` clone.
- A bundle that hooks only the hook points does not pin a Nightly: `hooks` in `compatibility.json`
  is the API version it needs, and `horizon_revision` is only "tested on".
- In-game proofs build a Linux binary here and run it with `horizon-ctl --offscreen` against the
  prepared resources above. The launcher's `native-build` is deliberately not an MCP tool: trusting
  native code is the player's decision.
- A native mod runs inside the game: a panic stops the game. Use `Option`/`Result`.