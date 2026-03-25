use crossterm::style::Color as CColor;
use ratatui::style::Color;

pub struct Theme {
    pub title_accent: Color,
    pub timestamp:    Color,
    pub detail_text:  Color,
    pub footer:       Color,
    pub tui_bg:       Color,
    pub tui_border:   Color,
    /// (bar_color, track_color) for Hour / Day / Week / Month / Year
    pub items: [(Color, Color); 5],
}

pub fn dark_theme() -> Theme {
    Theme {
        title_accent: Color::Rgb(180, 140, 255),
        timestamp:    Color::Rgb(100, 100, 120),
        detail_text:  Color::Rgb(140, 140, 160),
        footer:       Color::Rgb(60,  60,  80),
        tui_bg:       Color::Rgb(16,  16,  20),
        tui_border:   Color::Rgb(140, 140, 160),
        items: [
            (Color::Rgb(0,   210, 210), Color::Rgb(0,   80,  80)),  // Hour  – cyan
            (Color::Rgb(90,  140, 255), Color::Rgb(30,  50,  100)), // Day   – blue
            (Color::Rgb(210, 90,  210), Color::Rgb(85,  30,  85)),  // Week  – magenta
            (Color::Rgb(230, 190, 0),   Color::Rgb(90,  75,  0)),   // Month – yellow
            (Color::Rgb(80,  210, 80),  Color::Rgb(25,  85,  25)),  // Year  – green
        ],
    }
}

pub fn light_theme() -> Theme {
    Theme {
        title_accent: Color::Rgb(100, 60,  200),
        timestamp:    Color::Rgb(110, 110, 130),
        detail_text:  Color::Rgb(80,  80,  100),
        footer:       Color::Rgb(160, 160, 180),
        tui_bg:       Color::Rgb(248, 248, 252),
        tui_border:   Color::Rgb(190, 190, 210),
        items: [
            (Color::Rgb(0,   150, 150), Color::Rgb(180, 230, 230)), // Hour  – teal
            (Color::Rgb(50,  100, 220), Color::Rgb(195, 215, 255)), // Day   – blue
            (Color::Rgb(160, 50,  160), Color::Rgb(230, 185, 230)), // Week  – purple
            (Color::Rgb(160, 120, 0),   Color::Rgb(250, 235, 170)), // Month – amber
            (Color::Rgb(30,  150, 30),  Color::Rgb(185, 235, 185)), // Year  – green
        ],
    }
}

/// Detect theme from $COLORFGBG (set by most terminal emulators).
/// Format is "fg;bg" where bg >= 8 typically means a light background.
/// Falls back to dark if unset or ambiguous.
pub fn detect_theme() -> Theme {
    if let Ok(val) = std::env::var("COLORFGBG") {
        if let Some(bg_str) = val.split(';').last() {
            if let Ok(bg) = bg_str.trim().parse::<u8>() {
                if bg >= 8 {
                    return light_theme();
                }
            }
        }
    }
    dark_theme()
}

pub fn to_ct(c: Color) -> CColor {
    match c {
        Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
        _ => CColor::White,
    }
}

pub fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}
