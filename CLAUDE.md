# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build --release          # compile optimized binary
cargo run --release            # run with default theme
cargo run --release -- --theme mocha  # run with a specific theme
ICON_PICKER_THEME=dracula cargo run   # theme via env var
cargo check                    # fast compile check
cargo clippy                   # lint
```

Available themes: `contrast` (default), `late`, `purple`, `mocha`, `gruvbox`, `dracula`.

There is no test suite. Use `cargo check` and `cargo clippy` to validate changes.

## Build Script Dependency

`build.rs` copies `glyphnames.json` (Nerd Font glyph metadata) into `src/icon_picker/` on first build. It looks for the file at `../late-sh/late-ssh/src/app/icon_picker/glyphnames.json`. If the sibling repo is absent, the build panics with instructions to copy the file manually.

## Architecture

**tuicon** is a TUI icon-picker that lets users browse and select emoji, kaomoji, Unicode characters, and Nerd Font glyphs. The selected icon is printed to stdout on confirm.

### Key design: `/dev/tty` backend

`main.rs` opens `/dev/tty` directly as the terminal backend instead of using stdin/stdout. This is intentional — it allows shell capture (`VAR=$(icon-picker)`) to work correctly because the TUI is isolated from the pipe.

### State (`src/icon_picker/mod.rs`)

`IconPickerState` is the single shared state struct passed to all render and event functions:

- `tab` — active `IconPickerTab` (All / Emoji / Kaomoji / Unicode / NerdFont)
- `search_query` — `TextArea` with full emacs keybindings and undo stack
- `selected_index` / `scroll_offset` — selectable-item index (excludes section headers) and vertical scroll
- `visible_height`, `list_inner`, `tabs_inner` — `Cell<>` values written during render, read during mouse-click hit-testing
- `last_click` — `(Instant, selectable_index)` for double-click detection (400 ms window)

### Two index systems in the icon list

The list mixes **section headers** and **icon entries**. Two index systems coexist:

- **Flat index** — counts every row including headers
- **Selectable index** — counts only icon entries (skips headers)

`picker.rs` provides `selectable_to_flat`, `flat_to_selectable`, and `entry_at_selectable` to convert between them. Mouse click → row coordinate is translated to a selectable index via these helpers.

### Rendering (`src/icon_picker/picker.rs`)

`render()` applies a 4-row vertical layout per frame:

| Row | Content |
|-----|---------|
| 0 | Tab bar with bullet-point indicators |
| 1 | Search box (`search › ` prefix) |
| 2 | Scrollable icon list with section headers |
| 3 | Page counter (e.g. `page 3/5`) |

The list is organized into `IconSection` groups. Sections for each tab:
- **All**: every section from Emoji, Kaomoji, Unicode, and NerdFont concatenated, with titles prefixed by category (e.g. "Emoji · Common Emoji") — built once in `IconCatalogData::load()` by cloning entries from the other catalogs
- **Emoji**: "Common Emoji" + "All Emoji"
- **Kaomoji**: single section of 43 hardcoded entries
- **Unicode**: "Common Unicode" + 5 named ranges (Box Drawing, Geometric Shapes, Arrows, Math Operators, Dingbats)
- **NerdFont**: "Common Nerd Font" + "All Nerd Font"

### Search and filtering (`src/icon_picker/catalog.rs`)

Filtering runs at render time via `filter_sections()` and resets `selected_index` when the query changes.

**Standard tabs** (Emoji, Kaomoji, NerdFont): case-insensitive substring match on entry name.

**Unicode tab** is special:
- Empty query → pre-defined ranges shown
- Single character → direct codepoint match
- `U+XXXX` / `0xXXXX` → hex codepoint lookup
- Otherwise → scans all 0x10FFFF codepoints for Unicode name matches (capped at 200 results)
- Results cached with a 16-entry LRU to avoid re-scanning on backspace

### Theme system (`src/theme.rs`)

Theme is stored in thread-local storage. Six palettes with hardcoded RGB values. Selected at startup via `--theme <name>` or `ICON_PICKER_THEME`; defaults to `contrast`.
