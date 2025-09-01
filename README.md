<h1 align="center">Clipvault</h1>

<p align="center">Clipboard history manager for Wayland, inspired by <a href="https://github.com/sentriz/cliphist">cliphist</a></p>

<p align="center">
  <img src="https://img.shields.io/badge/License-AGPL_v3-green.svg" alt="License: AGPL v3" />
  <img src="https://img.shields.io/github/v/tag/rolv-apneseth/clipvault?label=version&color=blueviolet" alt="version" />
  <a href="https://crates.io/crates/clipvault"><img alt="Crates.io Version" src="https://img.shields.io/crates/v/clipvault"></a>
  <a href="https://aur.archlinux.org/packages/clipvault"><img src="https://img.shields.io/aur/version/clipvault" alt="AUR version" /></a>
</p>

<video alt="Clipvault demo" src="https://github.com/user-attachments/assets/8b8ddc27-faa1-443c-a948-840cd4fcc038"></video>

## Features

Like `cliphist`:

- **Save history:** clipboard entries are stored in a local database
- **Recall history:** recall saved entries using any picker you like (e.g. `dmenu`, `rofi`, `wofi`, etc.)
- **Simple:** no builtin picker, only pipes - keep it simple (stupid)
- **Any data:** support for **text**, **images** and **any other** binary data
- **Preservation:** entries are preserved byte-for-byte, including leading/trailing whitespace

In addition:

- **No silent failures:** invalid arguments cause errors, and are also written to log files
- **Relative positions:** support for getting/deleting items by relative position in the saved history
- **Entry size limits**: configurable minimum and maximum size for stored entries
- **Entry age limit:** configurable max age for entries - automatically remove old clipboard entries
- **Informative previews:** previews for binary data support many more types e.g. `video/mp4`, `application/pdf`, etc.

## Requirements

- [wl-clipboard](https://github.com/bugaevc/wl-clipboard), or anything with an interface
  equivalent to `wl-clipboard --watch` to keep `clipvault` updated with the latest
  clipboard entries.

## Installation

### Cargo

```bash
cargo install clipvault --locked
```

Or, directly from source:

```bash
cargo install --git https://github.com/rolv-apneseth/clipvault --locked
```

### AUR

```bash
paru -S clipvault
```

### Manual

1. Download the tarball for your computer's architecture (probably `x86_64`) from the [releases page](https://github.com/Rolv-Apneseth/clipvault/releases)
2. Unpack the tarball, e.g.:

    ```sh
    tar -xf clipvault-x86_64-unknown-linux-gnu.tar.gz
    ```

3. Place the `clipvault` binary in your `$PATH`

## Setup

```sh
wl-paste --watch clipvault store
```

This will listen for changes from the Wayland clipboard and write each entry to the history.
Call it once per session - for example:

- **Hyprland**:

    ```conf
    exec-once = wl-paste --watch clipvault store
    ```

- **Sway:**

    ```conf
    exec wl-paste --watch clipvault store
    ```

### Filtering

If you wish to narrow down the MIME types copied to `clipvault`, you can run `wl-paste --watch`
once for each type that should actually get forwarded to `clipvault`:

```sh
wl-paste --type text --watch clipvault store # Forward text data
wl-paste --type image --watch clipvault store # Forward raw image data
```

> [!TIP]
> Use `wl-paste --list-types` to find the available MIME types for the currently copied data in the
> Wayland clipboard.

### Image data from browsers

When copying images from browsers, `wl-paste` will usually pass the data to `clipvault` as `text/html`.
This is not ideal for copying images, and you may wish to have the raw image data copied instead.
If so, you can either *only* forward image data, or, more realistically, use the below and filter
out entries which start with `<meta http-equiv=` in your picker (check out some of the scripts
in [extras](./extras) - I personally use the [rofi script](./extras/clipvault_rofi.sh)):

```sh
wl-paste --watch clipvault store # Forward all data 
wl-paste --type image --watch clipvault store # Forward specifically raw image data 
```

## Usage

#### Select an entry (picker)

```sh
clipvault list | dmenu | clipvault get | wl-copy
```

I recommend making a keybind for this one with your favourite picker (see [picker examples](#picker-examples) below).

#### Select an entry (relative index)

```sh
clipvault get --index 0 # Newest entry
clipvault get --index 1 # Entry just before the newest entry 
clipvault get --index -1 # Oldest entry
```

#### Delete an entry (picker)

```sh
clipvault list | dmenu | clipvault delete
```

The `rofi` script in [extras](./extras), which creates a custom mode, uses `clipvault` delete to
remove entries directly from within the `rofi` window.

#### Delete an entry (relative index)

```sh
clipvault delete --index 0 # Delete the newest entry
clipvault delete --index 1 # Delete the entry just before the newest entry
clipvault delete --index -1 # Delete the oldest entry
```

#### Delete all entries

```sh
clipvault clear
```

Alternatively, just delete the database file (default path can be found in `help` output).

#### Additional information

- Logs are written to `$XDG_STATE_HOME/clipvault/logs`
- Log level can be changed using the `RUST_LOG` env var, e.g. `RUST_LOG="trace"`
- ANSI colours used for errors printed to STDOUT can be disabled with `NO_COLOR=1`

## Picker Examples

Examples of basic setups for different picker programs, such as `rofi` and `dmenu`.

Many of the below examples will show only the second column of the output from `clipvault list`
(each line is separated by `\t`). Like
`cliphist`, it's important that a line prefixed with a number is piped into `clipvault get`. This
number is used to look up in the database the exact original selection that was made, with all
leading/trailing/non-printable whitespace preserved, none of which is shown in the preview output of
`clipvault list`.

<details>
<summary>dmenu</summary>

```sh
clipvault list | dmenu | clipvault get | wl-copy
```

</details>

<details>
<summary>fzf</summary>

```sh
clipvault list | fzf --no-sort -d $'\t' --with-nth 2 | clipvault get | wl-copy
```

</details>

<details>
<summary>rofi</summary>

```sh
clipvault list | rofi -dmenu -display-columns 2 | clipvault get | wl-copy
```

</details>

<details>
<summary>fuzzel</summary>

```sh
clipvault list | fuzzel --dmenu --with-nth 2 | clipvault get | wl-copy
```

</details>

<details>
<summary>wofi</summary>

```sh
clipvault list | wofi -S dmenu --pre-display-cmd "echo '%s' | cut -f 2" | clipvault get | wl-copy
```

Note also that by default `wofi` may sort entries alphabetically or by its cache. To display entries
in the order they are produced, try adding the following arguments to the `wofi` call above:

```sh
-d -k /dev/null
```

</details>

<details>
<summary>tofi</summary>

```sh
clipvault list | tofi | clipvault get | wl-copy
```

</details>

For more advanced setups, checkout some of the scripts in the [extras](./extras/) directory.
Contributions are welcome.

> [!TIP]
> To avoid proceeding with copying if no entry was selected in the picker, try replacing
> `clipvault get | wl-copy` from any of the above commands with
> `{ read -r output && clipvault get <<< "$output" | wl-copy }`

## Configuration

While there is no support for a dedicated configuration file, `clipvault` *does* support loading additional
CLI arguments from files, thanks to [argfile](https://docs.rs/argfile/latest/argfile/).

To use this functionality, create a file with one argument per line, like [this example](./extras/argfile.txt).

Then, provide its path to the CLI using:

```sh
clipvault @/path/to/argfile
```

> [!TIP]
> Most options can also be set using environment variables - check out the `help` output
> for each command to find the specific variable to set for each.

## Contributing

All contributions are welcome. If you run into any problems, or have any suggestions/feedback, feel free to open an issue.

This project is written in Rust, so for contributing code:

1. Ensure [rustup](https://rustup.rs/) is installed - this project uses the stable toolchain for
   most things, but nightly for the formatting.
2. Make your changes and ensure they work as expected - `cargo run -- your_args_here`.
3. Lint + format + run tests:

    ```rust
    cargo clippy --all -- -W clippy::all && cargo +nightly fmt && cargo test
    ```

4. LGTM 👍

I like [just](https://github.com/casey/just), so I keep some utility commands in the [justfile](./justfile).
Check that out for additional checks which are run in the CI.

## Similar programs

- [cliphist](https://github.com/sentriz/cliphist)
- [cliprust](https://github.com/aulimaru/cliprust)

## Acknowledgements

- [cliphist](https://github.com/sentriz/cliphist) of course
- All the [dependencies](./Cargo.toml) of this project, and all [their dependencies](./Cargo.lock) too

## License

This code is licensed under the [AGPLv3](https://www.gnu.org/licenses/agpl-3.0.en.html#license-text).

See the [LICENSE](./LICENSE) file for more details.
