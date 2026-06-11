use ratatui::style::Color;
use std::cell::Cell;

struct Palette {
    bg_canvas: Color,
    bg_selection: Color,
    bg_highlight: Color,
    border_dim: Color,
    border_active: Color,
    text_faint: Color,
    text_dim: Color,
    text_muted: Color,
    text: Color,
    text_bright: Color,
    amber_dim: Color,
    amber_glow: Color,
}

const PALETTE_CONTRAST: Palette = Palette {
    bg_canvas: Color::Rgb(12, 14, 12),
    bg_selection: Color::Rgb(26, 30, 38),
    bg_highlight: Color::Rgb(34, 40, 50),
    border_dim: Color::Rgb(74, 84, 98),
    border_active: Color::Rgb(122, 201, 255),
    text_faint: Color::Rgb(126, 138, 155),
    text_dim: Color::Rgb(164, 176, 193),
    text_muted: Color::Rgb(194, 205, 220),
    text: Color::Rgb(226, 234, 245),
    text_bright: Color::Rgb(248, 251, 255),
    amber_dim: Color::Rgb(214, 160, 75),
    amber_glow: Color::Rgb(255, 216, 127),
};

const PALETTE_LATE: Palette = Palette {
    bg_canvas: Color::Rgb(0, 0, 0),
    bg_selection: Color::Rgb(30, 25, 22),
    bg_highlight: Color::Rgb(40, 33, 28),
    border_dim: Color::Rgb(50, 42, 36),
    border_active: Color::Rgb(160, 105, 42),
    text_faint: Color::Rgb(78, 65, 54),
    text_dim: Color::Rgb(105, 88, 72),
    text_muted: Color::Rgb(138, 118, 96),
    text: Color::Rgb(175, 158, 138),
    text_bright: Color::Rgb(200, 182, 158),
    amber_dim: Color::Rgb(130, 88, 38),
    amber_glow: Color::Rgb(210, 148, 54),
};

const PALETTE_PURPLE: Palette = Palette {
    bg_canvas: Color::Rgb(55, 57, 76),
    bg_selection: Color::Rgb(44, 26, 66),
    bg_highlight: Color::Rgb(58, 35, 84),
    border_dim: Color::Rgb(92, 72, 122),
    border_active: Color::Rgb(255, 171, 247),
    text_faint: Color::Rgb(176, 157, 199),
    text_dim: Color::Rgb(201, 184, 222),
    text_muted: Color::Rgb(220, 207, 236),
    text: Color::Rgb(236, 226, 248),
    text_bright: Color::Rgb(248, 242, 255),
    amber_dim: Color::Rgb(200, 140, 220),
    amber_glow: Color::Rgb(240, 180, 255),
};

const PALETTE_MOCHA: Palette = Palette {
    bg_canvas: Color::Rgb(30, 30, 46),
    bg_selection: Color::Rgb(69, 71, 90),
    bg_highlight: Color::Rgb(24, 24, 37),
    border_dim: Color::Rgb(49, 50, 68),
    border_active: Color::Rgb(203, 166, 247),
    text_faint: Color::Rgb(108, 112, 134),
    text_dim: Color::Rgb(147, 153, 178),
    text_muted: Color::Rgb(166, 173, 200),
    text: Color::Rgb(205, 214, 244),
    text_bright: Color::Rgb(245, 224, 220),
    amber_dim: Color::Rgb(200, 129, 35),
    amber_glow: Color::Rgb(249, 226, 175),
};

const PALETTE_GRUVBOX: Palette = Palette {
    bg_canvas: Color::Rgb(40, 40, 40),
    bg_selection: Color::Rgb(60, 56, 54),
    bg_highlight: Color::Rgb(29, 32, 33),
    border_dim: Color::Rgb(80, 73, 69),
    border_active: Color::Rgb(214, 93, 14),
    text_faint: Color::Rgb(146, 131, 116),
    text_dim: Color::Rgb(168, 153, 132),
    text_muted: Color::Rgb(189, 174, 147),
    text: Color::Rgb(235, 219, 178),
    text_bright: Color::Rgb(251, 241, 199),
    amber_dim: Color::Rgb(175, 124, 12),
    amber_glow: Color::Rgb(250, 189, 47),
};

const PALETTE_DRACULA: Palette = Palette {
    bg_canvas: Color::Rgb(40, 42, 54),
    bg_selection: Color::Rgb(68, 71, 90),
    bg_highlight: Color::Rgb(33, 34, 44),
    border_dim: Color::Rgb(68, 71, 90),
    border_active: Color::Rgb(189, 147, 249),
    text_faint: Color::Rgb(98, 114, 164),
    text_dim: Color::Rgb(139, 151, 189),
    text_muted: Color::Rgb(248, 248, 242),
    text: Color::Rgb(248, 248, 242),
    text_bright: Color::Rgb(255, 121, 198),
    amber_dim: Color::Rgb(191, 200, 90),
    amber_glow: Color::Rgb(241, 250, 140),
};

#[derive(Clone, Copy)]
pub enum Theme {
    Contrast,
    Late,
    Purple,
    Mocha,
    Gruvbox,
    Dracula,
}

impl Theme {
    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "late" => Self::Late,
            "purple" => Self::Purple,
            "mocha" => Self::Mocha,
            "gruvbox" => Self::Gruvbox,
            "dracula" => Self::Dracula,
            _ => Self::Contrast,
        }
    }

    pub fn names() -> &'static [&'static str] {
        &["contrast", "late", "purple", "mocha", "gruvbox", "dracula"]
    }
}

thread_local! {
    static CURRENT: Cell<Theme> = const { Cell::new(Theme::Contrast) };
}

pub fn set(theme: Theme) {
    CURRENT.with(|c| c.set(theme));
}

fn current() -> &'static Palette {
    CURRENT.with(|c| match c.get() {
        Theme::Contrast => &PALETTE_CONTRAST,
        Theme::Late => &PALETTE_LATE,
        Theme::Purple => &PALETTE_PURPLE,
        Theme::Mocha => &PALETTE_MOCHA,
        Theme::Gruvbox => &PALETTE_GRUVBOX,
        Theme::Dracula => &PALETTE_DRACULA,
    })
}

#[allow(non_snake_case)]
pub fn BG_CANVAS() -> Color {
    current().bg_canvas
}

#[allow(non_snake_case)]
pub fn BG_SELECTION() -> Color {
    current().bg_selection
}

#[allow(non_snake_case)]
pub fn BG_HIGHLIGHT() -> Color {
    current().bg_highlight
}

#[allow(non_snake_case)]
pub fn BORDER_DIM() -> Color {
    current().border_dim
}

#[allow(non_snake_case)]
pub fn BORDER_ACTIVE() -> Color {
    current().border_active
}

#[allow(non_snake_case)]
pub fn TEXT_FAINT() -> Color {
    current().text_faint
}

#[allow(non_snake_case)]
pub fn TEXT_DIM() -> Color {
    current().text_dim
}

#[allow(non_snake_case)]
pub fn TEXT_MUTED() -> Color {
    current().text_muted
}

#[allow(non_snake_case)]
pub fn TEXT() -> Color {
    current().text
}

#[allow(non_snake_case)]
pub fn TEXT_BRIGHT() -> Color {
    current().text_bright
}

#[allow(non_snake_case)]
pub fn AMBER_DIM() -> Color {
    current().amber_dim
}

#[allow(non_snake_case)]
pub fn AMBER_GLOW() -> Color {
    current().amber_glow
}
