# 😴 latuicon

`latuicon`, the **lat**e **TUI** **icon** picker: a rip-off of the [late.sh](https://github.com/mpiorowski/late-sh) embedded icon picker.

<p align="center">
    <img width="694" height="692" alt="latuicon-demo" src="https://github.com/user-attachments/assets/62ee3bb8-5870-4f97-96b0-d5571e64fea0" />
</p>

<p align="center">
    <a href="https://crates.io/crates/latuicon"><img src="https://img.shields.io/crates/v/latuicon.svg" alt="Crates info"></a>
    <a href="LICENSE"><img src="https://img.shields.io/github/license/coko7/latuicon?color=blue" alt="License: MIT"></a>
    <img src="https://img.shields.io/github/languages/top/coko7/latuicon?color=orange" alt="Rust">
    <a href="https://github.com/coko7/latuicon/actions/workflows/rust.yml"><img src="https://github.com/coko7/latuicon/actions/workflows/rust.yml/badge.svg" alt="Tests"></a>
</p>

A terminal UI icon picker for emoji, kaomoji, Unicode characters, and Nerd Font glyphs. Press Enter to print the selected icon to stdout; press Esc to exit without output.

## What's different from late.sh

The [initial commit](https://github.com/coko7/latuicon/commit/0fc806c) bootstrapped from the icon-picker component of [late.sh](https://github.com/mpiorowski/late-sh) (specifically code up until [6c670683](https://github.com/mpiorowski/late-sh/commit/6c670683e301cbef3df08683c84bc91141a0faee)). All development since then is independent.

Changes and additions:

- **Fuzzy search** 🪄 — word-level Levenshtein matching so small typos still find the right icon (e.g. `tumbs up` → 👍)
- **Data files** 📂 — emoji, kaomoji, unicode, and nerd font data extracted into editable JSON files under `data/`
- **Centered layout** 🎯 — subtitle under the title, icon-set block sized to content, search and icon list horizontally inset
- **Standalone binary** 📦 — packaged as `latuicon`, decoupled from the late.sh monorepo

Designed for shell capture:

```sh
icon=$(latuicon)
```

## Install

```sh
cargo install latuicon
```

### From source

```sh
./scripts/deploy.sh
```

Builds a release binary and installs it to `~/.local/bin/latuicon`.

## Usage

```sh
latuicon                          # default theme
latuicon --theme mocha            # specific theme
ICON_PICKER_THEME=dracula latuicon
```

Prints the chosen icon to stdout on confirm, nothing on Esc/Ctrl+C.

### Desktop integration example ([Hyprland](https://hypr.land/))

In my setup, I use [`floatty.sh`](https://github.com/coko7/scripts/blob/main/global/floatty.sh) to open `latuicon` in a floating terminal window and pipe the result to the clipboard. Here is my custom Hyprland binding for it:

```ini
bindd = $mainMod, comma, laTUIcon icon picker, exec, FLOATTY_CAPTURE_OUTPUT=1 bash floatty.sh latuicon latuicon | wl-copy
```

And these are the Hyprland window rules for it:

```ini
# Special rules for floating/prompt terminals
windowrule {
    name = floater-kitty
    match:class = ^(floater-kitty-.*)$
    no_anim = on
    float = on
    center = on
    size = 1000 800
}

windowrule {
  name = floater-kitty-latuicon
  match:class = floater-kitty-latuicon
  size = 700 700
}
```

Pressing `$mainMod + ,` opens a floating terminal with the picker; confirming an icon copies it straight to the Wayland clipboard.

## Keybindings

| Key | Action |
|-----|--------|
| `↑` / `↓` | Navigate list |
| `Ctrl+K` / `Ctrl+J` | Navigate list (vi-style) |
| `PgUp` / `PgDn` | Page up / down |
| `Ctrl+U` / `Ctrl+D` | Half-page up / down |
| `Tab` / `Shift+Tab` | Switch tab |
| `Enter` | Select and exit |
| `Esc` / `Ctrl+C` | Exit without selecting |
| Type anything | Filter by name |
| `Ctrl+Z` | Undo search edit |
| Mouse click | Select tab or item |
| Double-click | Select and exit |
| Scroll wheel | Scroll list |

Search supports full emacs cursor movement (`Ctrl+A`, `Ctrl+E`, `Ctrl+F`, `Ctrl+B`, `Ctrl+W`, `Ctrl+Y`, etc.).

## Themes

`contrast` (default), `late`, `purple`, `mocha`, `gruvbox`, `dracula`

## Tabs

- **All** — every icon from every other tab, combined into one searchable set
- **Emoji** — common emoji + full emoji set
- **Kaomoji** — curated kaomoji collection
- **Unicode** — common symbols + Box Drawing, Geometric Shapes, Arrows, Math Operators, Dingbats; search supports `U+XXXX` / `0xXXXX` hex lookup and full Unicode name scan
- **Nerd Font** — common glyphs + full Nerd Font glyph set

## Credits

The project was seeded from the icon-picker component of [late.sh](https://github.com/mpiorowski/late-sh) at commit [6c670683](https://github.com/mpiorowski/late-sh/commit/6c670683e301cbef3df08683c84bc91141a0faee). Code written after the initial commit is not derived from that project.

The original icon picker was written by [@mevanlc](https://github.com/mevanlc); the late.sh project is maintained by [@mpiorowski](https://github.com/mpiorowski).

See [`THIRD_PARTY_LICENSES.md`](./THIRD_PARTY_LICENSES.md) for the license covering the derived code from the initial commit.
