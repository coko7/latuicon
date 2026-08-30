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

use icon_picker::catalog::{IconCatalogData, SearchMode};
use icon_picker::{IconPickerState, IconPickerTab, picker};

/// interactive TUI icon/emoji/kaomoji picker
///
/// Prints the selected icon to stdout on Enter, nothing on Esc.
/// Useful in shell scripts: VAR=$(latuicon)
#[derive(Parser)]
#[command(
    name = "latuicon",
    version,
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
    /// Color theme
    #[arg(short = 't', long, env = "LATUICON_THEME", value_enum)]
    theme: Option<theme::Theme>,

    /// Icon tab shown on startup
    #[arg(
        short = 'd',
        long = "default-tab",
        env = "LATUICON_DEFAULT_TAB",
        value_enum,
        value_name = "TAB"
    )]
    default_tab: Option<IconPickerTab>,

    /// simple string match or fuzzy/typo-tolerant comparison
    #[arg(
        short = 's',
        long,
        env = "LATUICON_SEARCH",
        value_enum,
        value_name = "MODE"
    )]
    search_mode: Option<SearchMode>,

    /// Path to config file
    #[arg(short = 'c', long, env = "LATUICON_CONFIG")]
    config: Option<PathBuf>,

    /// Comma-separated list of enabled tabs, in display order
    #[arg(short = 'T', long, env = "LATUICON_TABS")]
    tabs: Option<String>,
}

const DEFAULT_THEME: theme::Theme = theme::Theme::Contrast;
const DEFAULT_TAB: IconPickerTab = IconPickerTab::Emoji;
const DEFAULT_SEARCH_MODE: SearchMode = SearchMode::Fuzzy;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let config = match Config::load(cli.config.as_deref()) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("latuicon: error: {err}");
            std::process::exit(1);
        }
    };

    let theme = cli.theme.or(config.theme).unwrap_or(DEFAULT_THEME);
    let default_tab = cli
        .default_tab
        .or(config.default_tab)
        .unwrap_or(DEFAULT_TAB);
    let search_mode = cli
        .search_mode
        .or(config.search_mode)
        .unwrap_or(DEFAULT_SEARCH_MODE);

    let cli_tabs = match cli.tabs {
        Some(raw) => {
            let names: Vec<String> = raw.split(',').map(|s| s.trim().to_string()).collect();
            match IconPickerTab::parse_tabs(&names) {
                Ok(tabs) => Some(tabs),
                Err(err) => {
                    eprintln!("latuicon: error: {err}");
                    std::process::exit(1);
                }
            }
        }
        None => None,
    };
    let tabs = cli_tabs
        .or(config.tabs)
        .unwrap_or_else(|| IconPickerTab::ALL.to_vec());

    if !tabs.contains(&default_tab) {
        eprintln!(
            "latuicon: error: default tab '{}' is disabled (not in tabs list)",
            default_tab.label()
        );
        std::process::exit(1);
    }

    theme::set(theme);

    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;

    enable_raw_mode()?;

    execute!(&tty, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(tty);
    let mut terminal = Terminal::new(backend)?;

    let catalog = IconCatalogData::load(&tabs);
    let mut state = IconPickerState::new(default_tab, search_mode, tabs);
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
        assert_eq!(cli.default_tab, None);
        assert_eq!(cli.search_mode, None);
    }

    #[test]
    fn cli_parses_theme_and_tab_flags() {
        let cli = Cli::parse_from(["latuicon", "--theme", "mocha", "--default-tab", "unicode"]);
        assert_eq!(cli.theme, Some(theme::Theme::Mocha));
        assert_eq!(cli.default_tab, Some(IconPickerTab::Unicode));
    }

    #[test]
    fn cli_accepts_short_flags_and_tab_alias() {
        let cli = Cli::parse_from(["latuicon", "-t", "dracula", "-d", "nerd"]);
        assert_eq!(cli.theme, Some(theme::Theme::Dracula));
        assert_eq!(cli.default_tab, Some(IconPickerTab::NerdFont));
    }

    #[test]
    fn cli_accepts_short_flag_for_tabs() {
        let cli = Cli::parse_from(["latuicon", "-T", "nerd,all,emoji"]);
        assert_eq!(cli.tabs, Some("nerd,all,emoji".to_string()));
    }

    #[test]
    fn cli_parses_search_mode_flag() {
        let cli = Cli::parse_from(["latuicon", "-s", "simple"]);
        assert_eq!(cli.search_mode, Some(SearchMode::Simple));
    }

    #[test]
    fn cli_rejects_unknown_search_mode() {
        assert!(Cli::try_parse_from(["latuicon", "--search-mode", "bogus"]).is_err());
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
        assert_eq!(
            None::<SearchMode>.or(None).unwrap_or(DEFAULT_SEARCH_MODE),
            DEFAULT_SEARCH_MODE
        );
    }

    #[test]
    fn cli_rejects_unknown_tab() {
        assert!(Cli::try_parse_from(["latuicon", "--default-tab", "bogus"]).is_err());
    }

    #[test]
    fn cli_rejects_unknown_theme() {
        assert!(Cli::try_parse_from(["latuicon", "--theme", "bogus"]).is_err());
    }

    #[test]
    fn cli_parses_tabs_flag() {
        let cli = Cli::parse_from(["latuicon", "--tabs", "nerd,all,emoji,kaomoji,unicode"]);
        assert_eq!(cli.tabs, Some("nerd,all,emoji,kaomoji,unicode".to_string()));
    }

    #[test]
    fn tabs_resolution_accepts_a_subset_to_disable_the_rest() {
        let names: Vec<String> = "all,emoji".split(',').map(str::to_string).collect();
        assert_eq!(
            IconPickerTab::parse_tabs(&names).unwrap(),
            vec![IconPickerTab::All, IconPickerTab::Emoji]
        );
    }

    #[test]
    fn tabs_resolution_accepts_full_permutation() {
        let names: Vec<String> = "nerd,all,emoji,kaomoji,unicode"
            .split(',')
            .map(str::to_string)
            .collect();
        assert_eq!(
            IconPickerTab::parse_tabs(&names).unwrap(),
            vec![
                IconPickerTab::NerdFont,
                IconPickerTab::All,
                IconPickerTab::Emoji,
                IconPickerTab::Kaomoji,
                IconPickerTab::Unicode,
            ]
        );
    }
}
