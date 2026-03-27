use chrono::NaiveDate;

use crate::theme::{Theme, dark_theme, detect_theme, light_theme};

pub struct Args {
    pub watch: bool,
    pub theme: Theme,
    pub birth: Option<NaiveDate>,
    pub lifespan: u32,
}

pub fn parse_args() -> Args {
    let raw: Vec<String> = std::env::args().collect();

    if raw.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        std::process::exit(0);
    }

    // Validate arguments
    let known_flags = [
        "--watch",
        "-w",
        "--theme",
        "--birth",
        "--lifespan",
        "-h",
        "--help",
    ];
    let known_value_flags = ["--theme", "--birth", "--lifespan"];
    let mut iter = raw.iter().skip(1).peekable();
    while let Some(arg) = iter.next() {
        if known_value_flags.contains(&arg.as_str()) {
            let missing = iter.peek().is_none_or(|v| v.starts_with('-'));
            if missing {
                eprintln!("error: `{arg}` requires a value\n");
                print_help();
                std::process::exit(1);
            }
            iter.next(); // consume value
        } else if !known_flags.contains(&arg.as_str()) {
            eprintln!("error: unknown argument `{arg}`\n");
            print_help();
            std::process::exit(1);
        }
    }

    let watch = raw.iter().any(|a| a == "--watch" || a == "-w");

    let theme_flag = raw
        .windows(2)
        .find(|w| w[0] == "--theme")
        .map(|w| w[1].as_str());

    let theme = match theme_flag {
        Some("dark") => dark_theme(),
        Some("light") => light_theme(),
        Some("auto") => detect_theme(),
        Some(other) => {
            eprintln!("error: unknown theme `{other}` (expected: dark | light | auto)\n");
            print_help();
            std::process::exit(1);
        }
        None => detect_theme(),
    };

    // Birth date: --birth flag takes priority over $HOURGLASS_BIRTH
    let birth_str = raw
        .windows(2)
        .find(|w| w[0] == "--birth")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("HOURGLASS_BIRTH").ok());

    let birth = birth_str.and_then(|s| NaiveDate::parse_from_str(&s, "%Y-%m-%d").ok());

    // Lifespan: --lifespan flag takes priority over $HOURGLASS_LIFESPAN, default 80
    let lifespan_str = raw
        .windows(2)
        .find(|w| w[0] == "--lifespan")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("HOURGLASS_LIFESPAN").ok());

    let lifespan = lifespan_str
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(80);

    Args {
        watch,
        theme,
        birth,
        lifespan,
    }
}

fn print_help() {
    println!(
        "\
hourglass — time progress visualization

USAGE
  hourglass [OPTIONS]

OPTIONS
  -w, --watch            live updating full-screen mode
      --theme THEME      color theme: dark | light | auto (default: auto)
      --birth YYYY-MM-DD birth date for life progress indicator
      --lifespan YEARS   expected lifespan in years (default: 80)
  -h, --help             show this help

ENVIRONMENT
  HOURGLASS_BIRTH        birth date (YYYY-MM-DD), enables life indicator
  HOURGLASS_LIFESPAN     expected lifespan in years (default: 80)"
    );
}
