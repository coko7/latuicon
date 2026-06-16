# 😴 latuicon

`latuicon`, the **lat**e **TUI** **icon** picker.

> [!NOTE]
> **This is a verbatim copy of the icon-picker component from [late.sh](https://github.com/mpiorowski/late-sh) by [@mpiorowski](https://github.com/mpiorowski), extracted into a standalone binary.**

A terminal UI icon picker for emoji, kaomoji, Unicode characters, and Nerd Font glyphs. Press Enter to print the selected icon to stdout; press Esc to exit without output.

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

All code is sourced from the [late.sh](https://github.com/mpiorowski/late-sh) project by [mpiorowski](https://github.com/mpiorowski). This repository only packages it as a standalone binary named `latuicon`.
