# 😴 latuicon

`latuicon`, the **lat**e **TUI** **icon** picker.

<img width="1007" height="808" alt="demo" src="https://github.com/user-attachments/assets/5946233c-672c-4f2f-8025-c7a02a665a15" />

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

- **Emoji** — common emoji + full emoji set
- **Kaomoji** — curated kaomoji collection
- **Unicode** — common symbols + Box Drawing, Geometric Shapes, Arrows, Math Operators, Dingbats; search supports `U+XXXX` / `0xXXXX` hex lookup and full Unicode name scan
- **Nerd Font** — common glyphs + full Nerd Font glyph set

## Credits

The project was seeded from the icon-picker component of [late.sh](https://github.com/mpiorowski/late-sh) by [@mpiorowski](https://github.com/mpiorowski) at commit [6c670683](https://github.com/mpiorowski/late-sh/commit/6c670683e301cbef3df08683c84bc91141a0faee). Code written after the initial commit is not derived from that project.
