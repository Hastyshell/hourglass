use std::io::{self, Write};
use std::time::Duration;

use chrono::{Datelike, Local, Timelike};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::style::{Color as CColor, Print, ResetColor, SetForegroundColor};
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Terminal;

const FILLED: &str = "━";
const EMPTY: &str = "─";
const HEAD: &str = "╸";

// ── Theme ─────────────────────────────────────────────────────────────────────

struct Theme {
    title_accent: Color,
    timestamp: Color,
    detail_text: Color,
    footer: Color,
    tui_bg: Color,
    tui_border: Color,
    // (bar_color, track_color) for Hour/Day/Week/Month/Year
    items: [(Color, Color); 5],
}

fn dark_theme() -> Theme {
    Theme {
        title_accent: Color::Rgb(180, 140, 255),
        timestamp:    Color::Rgb(100, 100, 120),
        detail_text:  Color::Rgb(140, 140, 160),
        footer:       Color::Rgb(60, 60, 80),
        tui_bg:       Color::Rgb(16, 16, 20),
        tui_border:   Color::Rgb(60, 60, 80),
        items: [
            (Color::Rgb(0,   210, 210), Color::Rgb(0,   50,  50)),  // Hour  – cyan
            (Color::Rgb(90,  140, 255), Color::Rgb(20,  30,  70)),  // Day   – blue
            (Color::Rgb(210, 90,  210), Color::Rgb(55,  20,  55)),  // Week  – magenta
            (Color::Rgb(230, 190, 0),   Color::Rgb(60,  50,  0)),   // Month – yellow
            (Color::Rgb(80,  210, 80),  Color::Rgb(15,  55,  15)),  // Year  – green
        ],
    }
}

fn light_theme() -> Theme {
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
fn detect_theme() -> Theme {
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

fn to_ct(c: Color) -> CColor {
    match c {
        Color::Rgb(r, g, b) => CColor::Rgb { r, g, b },
        _ => CColor::White,
    }
}

// ── Args ──────────────────────────────────────────────────────────────────────

struct Args {
    watch: bool,
    theme: Theme,
}

fn parse_args() -> Args {
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

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() -> io::Result<()> {
    let args = parse_args();
    if args.watch {
        run_tui(&args.theme)
    } else {
        print_inline(&args.theme)
    }
}

// ── Data ──────────────────────────────────────────────────────────────────────

struct ProgressItem {
    label:     &'static str,
    fraction:  f64,
    detail:    String,
    color:     Color,
    dim_color: Color,
}

fn get_progress_items(theme: &Theme) -> Vec<ProgressItem> {
    let now = Local::now();
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let hour   = now.hour()   as f64;

    let hour_frac = (minute * 60.0 + second) / 3600.0;

    let day_frac = (hour * 3600.0 + minute * 60.0 + second) / 86400.0;

    let weekday    = now.weekday().num_days_from_monday() as f64;
    let day_secs   = hour * 3600.0 + minute * 60.0 + second;
    let week_frac  = (weekday * 86400.0 + day_secs) / (7.0 * 86400.0);

    let day_of_month  = now.day() as f64 - 1.0;
    let days_in_month = days_in_current_month(&now);
    let month_frac    = (day_of_month * 86400.0 + day_secs) / (days_in_month as f64 * 86400.0);

    let day_of_year = now.ordinal() as f64 - 1.0;
    let days_in_year = if now.naive_local().date().leap_year() { 366.0 } else { 365.0 };
    let year_frac   = (day_of_year * 86400.0 + day_secs) / (days_in_year * 86400.0);

    let wday  = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let month = ["", "Jan", "Feb", "Mar", "Apr", "May", "Jun",
                     "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

    let fracs   = [hour_frac, day_frac, week_frac, month_frac, year_frac];
    let labels  = ["Hour", "Day", "Week", "Month", "Year"];
    let details = [
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()),
        format!("{} {:02}:{:02}", wday[now.weekday().num_days_from_monday() as usize],
                now.hour(), now.minute()),
        format!("{}  {} ∕ 7", wday[now.weekday().num_days_from_monday() as usize],
                now.weekday().num_days_from_monday() + 1),
        format!("{}  {} ∕ {}", month[now.month() as usize], now.day(), days_in_month),
        format!("{}  {} ∕ {:.0}", now.year(), now.ordinal(), days_in_year),
    ];

    labels.iter().enumerate().map(|(i, &label)| ProgressItem {
        label,
        fraction:  fracs[i],
        detail:    details[i].clone(),
        color:     theme.items[i].0,
        dim_color: theme.items[i].1,
    }).collect()
}

fn days_in_current_month(dt: &chrono::DateTime<Local>) -> u32 {
    let (year, month) = (dt.year(), dt.month());
    let next = if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap();
    next.signed_duration_since(chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap())
        .num_days() as u32
}

// ── Inline mode ───────────────────────────────────────────────────────────────

fn print_inline(theme: &Theme) -> io::Result<()> {
    let term_width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80);
    let width = term_width.min(72);
    let now   = Local::now();
    let items = get_progress_items(theme);
    let mut out = io::stdout();

    let (ta_r, ta_g, ta_b) = rgb(theme.title_accent);
    let (ts_r, ts_g, ts_b) = rgb(theme.timestamp);
    let (ft_r, ft_g, ft_b) = rgb(theme.footer);

    execute!(
        out,
        SetForegroundColor(CColor::Rgb { r: ta_r, g: ta_g, b: ta_b }),
        Print("⏳ hourglass"),
        SetForegroundColor(CColor::Rgb { r: ts_r, g: ts_g, b: ts_b }),
        Print(format!("  {}\n\n", now.format("%Y-%m-%d %H:%M:%S"))),
        ResetColor,
    )?;

    for item in &items {
        print_inline_item(&mut out, item, theme, width)?;
    }

    execute!(
        out,
        SetForegroundColor(CColor::Rgb { r: ft_r, g: ft_g, b: ft_b }),
        Print("-w / --watch        live updating mode\n--theme dark|light  color theme\n"),
        ResetColor,
    )?;

    out.flush()
}

fn print_inline_item(
    out: &mut impl Write,
    item: &ProgressItem,
    theme: &Theme,
    width: usize,
) -> io::Result<()> {
    let pct_str   = format!("{:.1}%", item.fraction * 100.0);
    let label_str = format!("{:<6}", item.label);
    let detail_w  = item.detail.chars().count();
    let pad       = width.saturating_sub(label_str.len() + detail_w + pct_str.len());
    let (dt_r, dt_g, dt_b) = rgb(theme.detail_text);

    execute!(
        out,
        SetForegroundColor(to_ct(item.color)),
        Print(&label_str),
        SetForegroundColor(CColor::Rgb { r: dt_r, g: dt_g, b: dt_b }),
        Print(&item.detail),
        Print(" ".repeat(pad)),
        SetForegroundColor(to_ct(item.color)),
        Print(&pct_str),
        Print("\n"),
    )?;

    let filled = ((width as f64) * item.fraction).floor() as usize;
    let filled = filled.min(width);

    if filled > 0 {
        execute!(out, SetForegroundColor(to_ct(item.color)), Print(FILLED.repeat(filled)))?;
    }
    if filled < width {
        execute!(out, SetForegroundColor(to_ct(item.color)), Print(HEAD))?;
        let remaining = width.saturating_sub(filled + 1);
        if remaining > 0 {
            execute!(out, SetForegroundColor(to_ct(item.dim_color)), Print(EMPTY.repeat(remaining)))?;
        }
    }

    execute!(out, Print("\n\n"), ResetColor)
}

// ── TUI / watch mode ──────────────────────────────────────────────────────────

fn run_tui(theme: &Theme) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend  = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| tui_ui(f, theme))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press
                    && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
                {
                    break;
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn tui_ui(f: &mut ratatui::Frame, theme: &Theme) {
    let size = f.area();

    f.render_widget(
        Block::default().style(Style::default().bg(theme.tui_bg)),
        size,
    );

    // border(2) + padding_top(1) + title(2) + 5×items(15) + footer(1) = 21
    // border(2) + padding_top(1) + title(2) + 5×items(15) + footer(2) = 22
    let content_height = 22;
    let content_width  = 72.min(size.width.saturating_sub(4));

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(content_height), Constraint::Min(0)])
        .split(size);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(content_width), Constraint::Min(0)])
        .split(vert[1]);

    let area  = horiz[1];
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.tui_border))
        .padding(Padding::new(2, 2, 1, 0))
        .style(Style::default().bg(theme.tui_bg));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // title
            Constraint::Length(3), // hour
            Constraint::Length(3), // day
            Constraint::Length(3), // week
            Constraint::Length(3), // month
            Constraint::Length(3), // year
            Constraint::Length(2), // footer
        ])
        .split(inner);

    let now = Local::now();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("⏳ ", Style::default().fg(theme.title_accent)),
            Span::styled("hourglass", Style::default().fg(theme.title_accent).bold()),
            Span::styled(
                format!("  {}", now.format("%Y-%m-%d %H:%M:%S")),
                Style::default().fg(theme.timestamp),
            ),
        ])),
        chunks[0],
    );

    let items = get_progress_items(theme);
    for (i, item) in items.iter().enumerate() {
        render_tui_item(f, chunks[i + 1], item, theme);
    }

    f.render_widget(
        Paragraph::new(vec![
            Line::from(Span::styled("q / Esc             quit", Style::default().fg(theme.footer))),
            Line::from(Span::styled("--theme dark|light  color theme", Style::default().fg(theme.footer))),
        ]),
        chunks[6],
    );
}

fn render_tui_item(f: &mut ratatui::Frame, area: Rect, item: &ProgressItem, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let pct_str   = format!("{:.1}%", item.fraction * 100.0);
    let label_str = format!("{:<6}", item.label);
    let detail_w  = item.detail.chars().count();
    let pad       = (area.width as usize)
        .saturating_sub(label_str.len() + detail_w + pct_str.len());

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(label_str, Style::default().fg(item.color).bold()),
            Span::styled(item.detail.clone(), Style::default().fg(theme.detail_text)),
            Span::raw(" ".repeat(pad)),
            Span::styled(pct_str, Style::default().fg(item.color)),
        ])),
        chunks[0],
    );

    let bar_width = area.width as usize;
    if bar_width == 0 {
        return;
    }

    let filled = ((bar_width as f64) * item.fraction).floor() as usize;
    let filled = filled.min(bar_width);
    let mut spans = Vec::new();

    if filled > 0 {
        spans.push(Span::styled(FILLED.repeat(filled), Style::default().fg(item.color)));
    }
    if filled < bar_width {
        spans.push(Span::styled(HEAD, Style::default().fg(item.color)));
        let remaining = bar_width.saturating_sub(filled + 1);
        if remaining > 0 {
            spans.push(Span::styled(
                EMPTY.repeat(remaining),
                Style::default().fg(item.dim_color),
            ));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), chunks[1]);
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn rgb(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => (255, 255, 255),
    }
}
