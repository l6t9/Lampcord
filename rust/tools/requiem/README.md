# requiem

Dev CLI. `cargo requiem <command>` (alias in `.cargo/config.toml`). The binary is `requiem`.

```sh
cargo requiem doctor
cargo requiem build
cargo requiem build -p requiem-ui
cargo requiem run
cargo requiem test
cargo requiem asio
cargo requiem cargo -- <cargo args>

cargo requiem assets resolve
cargo requiem assets show
cargo requiem assets inventory
cargo requiem assets inventory mp3
cargo requiem assets check
cargo requiem assets find "M40 29C37.79"
cargo requiem assets svg <Name>

cargo requiem emoji vendors
cargo requiem emoji build apple
cargo requiem emoji manifest
```

Also `asio` (fetches the Steinberg SDK into `vendor/ASIOSDK`), `desktop`, `fingerprint`, `plugin` (`new`, `build`), and `release` (`keygen`, `sign`, `resign`, `identity`, `publish`, `keys-json`, `verify`).

The first `cargo requiem` compiles the CLI, so it needs a working linker. After that, `target/debug/requiem.exe` runs directly. It embeds the recipe list, so rebuild after editing `recipes.rs`. On Windows, copy the exe before rebuilding it by hand: a running exe cannot overwrite itself.

## Build environment

On Windows a plain shell is missing MSVC's `INCLUDE`/`LIB`, MSVC's `link.exe` ahead of Git for Windows' `link.exe`, and `FFMPEG_DIR` for the `video` feature. `build`, `run`, `test`, and `cargo` set those for the child process only. `doctor` prints what it found. Other platforms do nothing.

On Windows, `build`, `run`, `test`, and `cargo` also fetch the Steinberg ASIO SDK into `vendor/ASIOSDK` when it is missing. `.cargo/config.toml` points `CPAL_ASIO_DIR` there, so an `asio` build fails until those headers exist. `asio` does the same fetch on its own. `doctor` reports whether the SDK is present.

`build --release` uses thin LTO and 16 codegen units, and it does not run Clippy afterward. The published binary is `cargo build --profile ship`: fat LTO, one codegen unit, stripped. That profile is for CI, not for a local check.

## Assets

Icons, fonts, Lottie, and illustrations are not in the repo. `assets/manifest.json`, embedded at compile time, lists each file on Discord's CDN with its hash. Recipes are in `crates/requiem-assets/src/recipes.rs`. The client reads them through `crates/requiem-asset-store`.

`assets resolve` downloads the current web build (cached under `.cache/discord-builds`, gitignored) and writes the manifest. It keeps entries the manifest already has. A new recipe is resolved. An old one stays pointed at the build it came from.

```sh
cargo requiem assets resolve
cargo requiem assets resolve --refresh icon/ --refresh lottie/mic
cargo requiem assets resolve --client keep
cargo requiem assets resolve --fresh
```

`--client` picks the `client_build_number` sent in `X-Super-Properties`: `same` (default), `keep`, `latest`, or a build id. A recipe that no longer resolves fails the command. Commit `assets/manifest.json` after a resolve. Set `REQUIEM_ASSET_MANIFEST_URL` to serve a corrected manifest without a new release. Unset or unreachable, the embedded one is used.

Recipe kinds are in `crates/requiem-assets/src/resolve.rs`.

Sounds are `sound/<name>` entries. `assets inventory mp3` lists the names. Play one with `requiem_core::sounds::play` (the `sound` feature).

To add an icon: `assets inventory icons` lists the components, `assets svg <Name>` prints the SVG, add `icon("icon/<Name>")` to `recipes.rs`, then `assets resolve`. The code constant drops `Icon` and uses screaming snake case, so `XSmallIcon` is `icons::X_SMALL`. Names match Discord's component names. An unexported icon is found by path data and named for what it is (`icon/replay`).

An icon is stored as the SVG the component renders. A color taken as a parameter is `currentColor`. The engine draws it through `usvg` (`requiem_ui::svg_icon`), so one asset scales to any size.
