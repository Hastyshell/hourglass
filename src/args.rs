use crate::theme::{Theme, dark_theme, detect_theme, light_theme};

pub struct Args {
    pub watch: bool,
    pub theme: Theme,
}

pub fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();
    let watch = raw.iter().any(|a| a == "--watch" || a == "-w");

    let theme_flag = raw.windows(2)
        .find(|w| w[0] == "--theme")
        .map(|w| w[1].as_str());

    let theme = match theme_flag {
        Some("dark")  => dark_theme(),
        Some("light") => light_theme(),
        _             => detect_theme(), // "auto" or unset
    };

    Args { watch, theme }
}
