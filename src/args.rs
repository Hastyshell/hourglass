use chrono::{NaiveDate, NaiveTime};

use crate::theme::{Theme, dark_theme, detect_theme, light_theme};

pub struct Args {
    pub watch: bool,
    pub theme: Theme,
    pub birth: Option<NaiveDate>,
    pub lifespan: u32,
    pub day_start: NaiveTime,
    pub day_end: NaiveTime,
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
        "--day-start",
        "--day-end",
        "-h",
        "--help",
    ];
    let known_value_flags = [
        "--theme",
        "--birth",
        "--lifespan",
        "--day-start",
        "--day-end",
    ];
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

    // Day window: --day-start/--day-end flags take priority over env vars.
    // When start == end, preserve the original full-day semantics.
    let day_start_str = raw
        .windows(2)
        .find(|w| w[0] == "--day-start")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("HOURGLASS_DAY_START").ok());
    let day_start = match &day_start_str {
        Some(s) => parse_time_value(s).unwrap_or_else(|| {
            eprintln!("warning: invalid day-start `{s}`, falling back to 00:00");
            midnight()
        }),
        None => midnight(),
    };

    let day_end_str = raw
        .windows(2)
        .find(|w| w[0] == "--day-end")
        .map(|w| w[1].clone())
        .or_else(|| std::env::var("HOURGLASS_DAY_END").ok());
    let day_end = match &day_end_str {
        Some(s) => parse_time_value(s).unwrap_or_else(|| {
            eprintln!("warning: invalid day-end `{s}`, falling back to 00:00");
            midnight()
        }),
        None => midnight(),
    };

    Args {
        watch,
        theme,
        birth,
        lifespan,
        day_start,
        day_end,
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
      --day-start HH:MM[:SS]  active day start time (default: 00:00)
      --day-end HH:MM[:SS]    active day end time (default: 00:00)
  -h, --help             show this help

ENVIRONMENT
  HOURGLASS_BIRTH        birth date (YYYY-MM-DD), enables life indicator
  HOURGLASS_LIFESPAN     expected lifespan in years (default: 80)
  HOURGLASS_DAY_START    active day start time (HH:MM or HH:MM:SS, default: 00:00)
  HOURGLASS_DAY_END      active day end time (HH:MM or HH:MM:SS, default: 00:00)"
    );
}

fn midnight() -> NaiveTime {
    NaiveTime::from_hms_opt(0, 0, 0).unwrap()
}

fn parse_time_value(value: &str) -> Option<NaiveTime> {
    ["%H:%M", "%H:%M:%S"]
        .iter()
        .find_map(|fmt| NaiveTime::parse_from_str(value, fmt).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_time_value_supports_hour_minute() {
        let parsed = parse_time_value("08:30").unwrap();
        assert_eq!(parsed, NaiveTime::from_hms_opt(8, 30, 0).unwrap());
    }

    #[test]
    fn parse_time_value_supports_seconds() {
        let parsed = parse_time_value("23:15:45").unwrap();
        assert_eq!(parsed, NaiveTime::from_hms_opt(23, 15, 45).unwrap());
    }

    #[test]
    fn parse_time_value_rejects_invalid_input() {
        assert!(parse_time_value("25:00").is_none());
    }
}
