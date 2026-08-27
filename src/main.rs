mod config;
mod icon_picker;
mod theme;

use clap::Parser;
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseButton,
        MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use ratatui_textarea::{Input, Key};
use std::path::PathBuf;
use std::{fs::OpenOptions, io};

use config::Config;

use icon_picker::catalog::IconCatalogData;
use icon_picker::{IconPickerState, IconPickerTab, picker};

/// interactive TUI icon/emoji/kaomoji picker
///
/// Prints the selected icon to stdout on Enter, nothing on Esc.
/// Useful in shell scripts: VAR=$(latuicon)
#[derive(Parser)]
#[command(
    name = "latuicon",
    after_help = "KEYS:\n  \
        ↑/↓          navigate list\n  \
        PgUp/PgDn    page up/down\n  \
        Ctrl+U/D     half-page up/down\n  \
        Tab/S+Tab    switch icon tab\n  \
        Enter        select and exit\n  \
        Esc/Ctrl+C   exit without selecting\n  \
        (type)       filter by name"
)]
struct Cli {
    /// Color theme (overrides config file)
    #[arg(short = 't', long, env = "LATUICON_THEME", value_enum)]
    theme: Option<theme::Theme>,

    /// Icon tab shown on startup (overrides config file)
    #[arg(short = 'T', long, env = "LATUICON_TAB", value_enum)]
    tab: Option<IconPickerTab>,

    /// Path to config file (defaults to $XDG_CONFIG_HOME/latuicon/config.toml,
    /// or ~/.config/latuicon/config.toml)
    #[arg(short = 'c', long, env = "LATUICON_CONFIG")]
    config: Option<PathBuf>,
}

/// Default theme when unset by CLI flag, env var, and config file.
const DEFAULT_THEME: theme::Theme = theme::Theme::Contrast;
/// Default startup tab when unset by CLI flag, env var, and config file.
const DEFAULT_TAB: IconPickerTab = IconPickerTab::Emoji;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref());

    let theme = cli.theme.or(config.theme).unwrap_or(DEFAULT_THEME);
    let tab = cli.tab.or(config.default_tab).unwrap_or(DEFAULT_TAB);

    theme::set(theme);

    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;

    enable_raw_mode()?;

    execute!(&tty, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(tty);
    let mut terminal = Terminal::new(backend)?;

    let mut state = IconPickerState::new(tab);
    let catalog = IconCatalogData::load();
    let mut selected: Option<String> = None;

    loop {
        terminal.draw(|f| {
            picker::render(f, f.area(), &state, &catalog);
        })?;

        match event::read()? {
            Event::Key(key) => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                let alt = key.modifiers.contains(KeyModifiers::ALT);

                match key.code {
                    // Exit
                    KeyCode::Esc => break,
                    KeyCode::Char('c') if ctrl => break,

                    // Confirm selection
                    KeyCode::Enter if !ctrl && !alt => {
                        if let Some(icon) = picker::selected_icon(&state, &catalog) {
                            selected = Some(icon);
                        }
                        break;
                    }

                    // List navigation — up
                    KeyCode::Up => picker::move_selection(&mut state, &catalog, -1),
                    KeyCode::Char('k') if ctrl => picker::move_selection(&mut state, &catalog, -1),

                    // List navigation — down
                    KeyCode::Down => picker::move_selection(&mut state, &catalog, 1),
                    KeyCode::Char('j') if ctrl => picker::move_selection(&mut state, &catalog, 1),

                    // Page navigation
                    KeyCode::PageUp => {
                        let h = state.visible_height.get() as isize;
                        picker::move_selection(&mut state, &catalog, -h);
                    }
                    KeyCode::PageDown => {
                        let h = state.visible_height.get() as isize;
                        picker::move_selection(&mut state, &catalog, h);
                    }
                    // Half-page navigation (Ctrl+U / Ctrl+D, same as the in-app picker)
                    KeyCode::Char('u') if ctrl => {
                        let half = (state.visible_height.get() / 2).max(1) as isize;
                        picker::move_selection(&mut state, &catalog, -half);
                    }
                    KeyCode::Char('d') if ctrl => {
                        let half = (state.visible_height.get() / 2).max(1) as isize;
                        picker::move_selection(&mut state, &catalog, half);
                    }

                    // Tab switching
                    KeyCode::Tab => state.next_tab(),
                    KeyCode::BackTab => state.prev_tab(),

                    // Search: cursor movement
                    KeyCode::Left if ctrl || alt => state.search_cursor_word_left(),
                    KeyCode::Right if ctrl || alt => state.search_cursor_word_right(),
                    KeyCode::Left => state.search_cursor_left(),
                    KeyCode::Right => state.search_cursor_right(),
                    KeyCode::Home => state.search_cursor_home(),
                    KeyCode::End => state.search_cursor_end(),

                    // Search: deletion
                    KeyCode::Backspace if ctrl => state.search_delete_word_left(),
                    KeyCode::Backspace => state.search_delete_char(),
                    KeyCode::Delete if ctrl => state.search_delete_word_right(),
                    KeyCode::Delete => state.search_delete_next_char(),
                    KeyCode::Char('w') if ctrl => state.search_delete_word_left(),

                    // Ctrl+Z → undo search edit
                    KeyCode::Char('z') if ctrl => state.search_undo(),

                    // Forward remaining Ctrl+letter chords to textarea emacs bindings
                    // (^A head-of-line, ^E end, ^F forward, ^B back, ^Y yank, etc.)
                    KeyCode::Char(ch) if ctrl && ch.is_ascii_lowercase() => {
                        state.search_input(Input {
                            key: Key::Char(ch),
                            ctrl: true,
                            alt: false,
                            shift: false,
                        });
                    }

                    // Printable characters → search box
                    KeyCode::Char(ch) if !ctrl && !alt => state.search_insert_char(ch),

                    _ => {}
                }
            }

            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::Down(MouseButton::Left) => {
                    if !picker::click_tab(&mut state, mouse.column, mouse.row) {
                        let confirmed =
                            picker::click_list(&mut state, &catalog, mouse.column, mouse.row);
                        if confirmed {
                            if let Some(icon) = picker::selected_icon(&state, &catalog) {
                                selected = Some(icon);
                            }
                            break;
                        }
                    }
                }
                MouseEventKind::ScrollUp => picker::move_selection(&mut state, &catalog, -3),
                MouseEventKind::ScrollDown => picker::move_selection(&mut state, &catalog, 3),
                _ => {}
            },

            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    // Restore terminal before printing output so it lands in normal shell context.
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(icon) = selected {
        print!("{icon}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_leaves_theme_and_tab_unset_when_omitted() {
        let cli = Cli::try_parse_from(["latuicon"]).unwrap();
        assert_eq!(cli.theme, None);
        assert_eq!(cli.tab, None);
    }

    #[test]
    fn cli_parses_theme_and_tab_flags() {
        let cli = Cli::parse_from(["latuicon", "--theme", "mocha", "--tab", "unicode"]);
        assert_eq!(cli.theme, Some(theme::Theme::Mocha));
        assert_eq!(cli.tab, Some(IconPickerTab::Unicode));
    }

    #[test]
    fn cli_accepts_short_flags_and_tab_alias() {
        let cli = Cli::parse_from(["latuicon", "-t", "dracula", "-T", "nerd"]);
        assert_eq!(cli.theme, Some(theme::Theme::Dracula));
        assert_eq!(cli.tab, Some(IconPickerTab::NerdFont));
    }

    #[test]
    fn resolved_theme_and_tab_fall_back_through_cli_config_default() {
        // CLI flag wins over config.
        assert_eq!(
            Some(theme::Theme::Dracula).or(Some(theme::Theme::Mocha)),
            Some(theme::Theme::Dracula)
        );
        // Config wins over the built-in default when CLI is unset.
        assert_eq!(
            None.or(Some(theme::Theme::Mocha)).unwrap_or(DEFAULT_THEME),
            theme::Theme::Mocha
        );
        // Built-in default applies when neither CLI nor config set it.
        assert_eq!(
            None::<theme::Theme>.or(None).unwrap_or(DEFAULT_THEME),
            DEFAULT_THEME
        );
        assert_eq!(
            None::<IconPickerTab>.or(None).unwrap_or(DEFAULT_TAB),
            DEFAULT_TAB
        );
    }

    #[test]
    fn cli_rejects_unknown_tab() {
        assert!(Cli::try_parse_from(["latuicon", "--tab", "bogus"]).is_err());
    }

    #[test]
    fn cli_rejects_unknown_theme() {
        assert!(Cli::try_parse_from(["latuicon", "--theme", "bogus"]).is_err());
    }
}
