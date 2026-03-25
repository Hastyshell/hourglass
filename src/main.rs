use std::io;
use std::time::Duration;

use chrono::{Datelike, Local, Timelike};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
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

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| ui(f))?;

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

struct ProgressItem {
    label: &'static str,
    fraction: f64,
    detail: String,
    color: Color,
    dim_color: Color,
}

fn get_progress_items() -> Vec<ProgressItem> {
    let now = Local::now();
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let hour = now.hour() as f64;

    // Hour progress
    let hour_frac = (minute * 60.0 + second) / 3600.0;

    // Day progress
    let day_frac = (hour * 3600.0 + minute * 60.0 + second) / 86400.0;

    // Week progress (Monday = 1)
    let weekday = now.weekday().num_days_from_monday() as f64; // 0=Mon
    let day_seconds = hour * 3600.0 + minute * 60.0 + second;
    let week_frac = (weekday * 86400.0 + day_seconds) / (7.0 * 86400.0);

    // Month progress
    let day_of_month = now.day() as f64 - 1.0;
    let days_in_month = days_in_current_month(&now);
    let month_frac = (day_of_month * 86400.0 + day_seconds) / (days_in_month as f64 * 86400.0);

    // Year progress
    let day_of_year = now.ordinal() as f64 - 1.0;
    let days_in_year = if now.naive_local().date().leap_year() {
        366.0
    } else {
        365.0
    };
    let year_frac = (day_of_year * 86400.0 + day_seconds) / (days_in_year * 86400.0);

    let weekday_names = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let month_names = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    vec![
        ProgressItem {
            label: "Hour",
            fraction: hour_frac,
            detail: format!(
                "{:02}:{:02}:{:02}",
                now.hour(),
                now.minute(),
                now.second()
            ),
            color: Color::Rgb(0, 210, 210),
            dim_color: Color::Rgb(0, 50, 50),
        },
        ProgressItem {
            label: "Day",
            fraction: day_frac,
            detail: format!(
                "{} {:02}:{:02}",
                weekday_names[now.weekday().num_days_from_monday() as usize],
                now.hour(),
                now.minute()
            ),
            color: Color::Rgb(90, 140, 255),
            dim_color: Color::Rgb(20, 30, 70),
        },
        ProgressItem {
            label: "Week",
            fraction: week_frac,
            detail: format!(
                "{}  {} ∕ 7",
                weekday_names[now.weekday().num_days_from_monday() as usize],
                now.weekday().num_days_from_monday() + 1,
            ),
            color: Color::Rgb(210, 90, 210),
            dim_color: Color::Rgb(55, 20, 55),
        },
        ProgressItem {
            label: "Month",
            fraction: month_frac,
            detail: format!(
                "{}  {} ∕ {}",
                month_names[now.month() as usize],
                now.day(),
                days_in_month
            ),
            color: Color::Rgb(230, 190, 0),
            dim_color: Color::Rgb(60, 50, 0),
        },
        ProgressItem {
            label: "Year",
            fraction: year_frac,
            detail: format!("{}  {} ∕ {:.0}", now.year(), now.ordinal(), days_in_year),
            color: Color::Rgb(80, 210, 80),
            dim_color: Color::Rgb(15, 55, 15),
        },
    ]
}

fn days_in_current_month(dt: &chrono::DateTime<Local>) -> u32 {
    let year = dt.year();
    let month = dt.month();
    if month == 12 {
        chrono::NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        chrono::NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap()
    .signed_duration_since(chrono::NaiveDate::from_ymd_opt(year, month, 1).unwrap())
    .num_days() as u32
}

fn ui(f: &mut ratatui::Frame) {
    let size = f.area();

    // Dark background
    f.render_widget(
        Block::default().style(Style::default().bg(Color::Rgb(16, 16, 20))),
        size,
    );

    // Center the content
    // border(2) + padding_top(1) + title(2) + 5×items(15) + footer(1) = 21
    let content_height = 21;
    let content_width = 72.min(size.width.saturating_sub(4));

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(content_height),
            Constraint::Min(0),
        ])
        .split(size);

    let horiz = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(content_width),
            Constraint::Min(0),
        ])
        .split(vert[1]);

    let area = horiz[1];

    // Outer border
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
        .padding(Padding::new(2, 2, 1, 0))
        .style(Style::default().bg(Color::Rgb(16, 16, 20)));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    // Layout: title + items + footer
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // title
            Constraint::Length(3), // hour
            Constraint::Length(3), // day
            Constraint::Length(3), // week
            Constraint::Length(3), // month
            Constraint::Length(3), // year
            Constraint::Length(1), // footer
        ])
        .split(inner);

    // Title
    let now = Local::now();
    let title = Line::from(vec![
        Span::styled("⏳ ", Style::default().fg(Color::Rgb(180, 140, 255))),
        Span::styled(
            "hourglass",
            Style::default()
                .fg(Color::Rgb(180, 140, 255))
                .bold(),
        ),
        Span::styled(
            format!("  {}", now.format("%Y-%m-%d %H:%M:%S")),
            Style::default().fg(Color::Rgb(100, 100, 120)),
        ),
    ]);
    f.render_widget(Paragraph::new(title), chunks[0]);

    // Progress items
    let items = get_progress_items();
    for (i, item) in items.iter().enumerate() {
        render_progress_item(f, chunks[i + 1], item);
    }

    // Footer
    let footer = Line::from(vec![Span::styled(
        "press q to quit",
        Style::default().fg(Color::Rgb(60, 60, 80)),
    )]);
    f.render_widget(Paragraph::new(footer), chunks[6]);
}

fn render_progress_item(f: &mut ratatui::Frame, area: Rect, item: &ProgressItem) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    let pct = item.fraction * 100.0;

    // Label line
    let label_line = Line::from(vec![
        Span::styled(
            format!("{:<6}", item.label),
            Style::default().fg(item.color).bold(),
        ),
        Span::styled(
            item.detail.clone(),
            Style::default().fg(Color::Rgb(140, 140, 160)),
        ),
        Span::styled(
            format!("{:>width$}", format!("{:.1}%", pct), width = (area.width as usize).saturating_sub(6 + item.detail.len())),
            Style::default().fg(item.color),
        ),
    ]);
    f.render_widget(Paragraph::new(label_line), chunks[0]);

    // Progress bar
    let bar_width = area.width as usize;
    if bar_width == 0 {
        return;
    }

    let filled_count = ((bar_width as f64) * item.fraction).floor() as usize;
    let filled_count = filled_count.min(bar_width);

    let dim_color = item.dim_color;

    let mut spans = Vec::new();

    if filled_count > 0 {
        spans.push(Span::styled(
            FILLED.repeat(filled_count),
            Style::default().fg(item.color),
        ));
    }

    if filled_count < bar_width {
        spans.push(Span::styled(HEAD, Style::default().fg(item.color)));
        let remaining = bar_width.saturating_sub(filled_count + 1);
        if remaining > 0 {
            spans.push(Span::styled(
                EMPTY.repeat(remaining),
                Style::default().fg(dim_color),
            ));
        }
    }

    f.render_widget(Paragraph::new(Line::from(spans)), chunks[1]);
}

