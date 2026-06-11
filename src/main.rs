mod icon_picker;
mod theme;

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
use std::io;

use icon_picker::catalog::IconCatalogData;
use icon_picker::{IconPickerState, picker};

fn main() -> io::Result<()> {
    let theme = parse_theme();
    theme::set(theme);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = IconPickerState::default();
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
        println!("{icon}");
    }

    Ok(())
}

fn parse_theme() -> theme::Theme {
    let args: Vec<String> = std::env::args().collect();

    let from_args = args
        .windows(2)
        .find(|w| w[0] == "--theme" || w[0] == "-t")
        .map(|w| w[1].clone());

    let name = from_args
        .or_else(|| std::env::var("ICON_PICKER_THEME").ok())
        .unwrap_or_default();

    if name == "--help" || args.iter().any(|a| a == "--help" || a == "-h") {
        eprintln!("icon-picker — interactive TUI icon/emoji/kaomoji picker");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("  icon-picker [--theme <name>]");
        eprintln!();
        eprintln!("  Prints the selected icon to stdout on Enter, nothing on Esc.");
        eprintln!("  Useful in shell scripts: VAR=$(icon-picker)");
        eprintln!();
        eprintln!("THEMES: {}", theme::Theme::names().join(", "));
        eprintln!("  Also: ICON_PICKER_THEME=<name> icon-picker");
        eprintln!();
        eprintln!("KEYS:");
        eprintln!("  ↑/↓          navigate list");
        eprintln!("  PgUp/PgDn    page up/down");
        eprintln!("  Ctrl+U/D     half-page up/down");
        eprintln!("  Tab/S+Tab    switch icon tab");
        eprintln!("  Enter        select and exit");
        eprintln!("  Esc/Ctrl+C   exit without selecting");
        eprintln!("  (type)       filter by name");
        std::process::exit(0);
    }

    theme::Theme::from_str(&name)
}
