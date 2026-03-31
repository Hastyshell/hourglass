use std::io;
use std::time::Duration;

use chrono::Local;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};

use chrono::{NaiveDate, NaiveTime};

use crate::progress::{ProgressItem, get_progress_items};
use crate::theme::Theme;
use crate::{EMPTY, FILLED, HEAD};

pub fn run_tui(
    theme: &Theme,
    birth: Option<NaiveDate>,
    lifespan: u32,
    day_start: NaiveTime,
    day_end: NaiveTime,
) -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        terminal.draw(|f| draw(f, theme, birth, lifespan, day_start, day_end))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
            && matches!(key.code, KeyCode::Char('q') | KeyCode::Esc)
        {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw(
    f: &mut ratatui::Frame,
    theme: &Theme,
    birth: Option<NaiveDate>,
    lifespan: u32,
    day_start: NaiveTime,
    day_end: NaiveTime,
) {
    let size = f.area();

    f.render_widget(
        Block::default().style(Style::default().bg(theme.tui_bg)),
        size,
    );

    let items = get_progress_items(theme, birth, lifespan, day_start, day_end);
    // border(2) + padding_top(1) + title(2) + n×items(3 each) + footer(1)
    let content_height = (2 + 1 + 2 + items.len() * 3 + 1) as u16;
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
    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(theme.tui_border))
        .padding(Padding::new(2, 2, 1, 0))
        .style(Style::default().bg(theme.tui_bg));

    let inner = outer.inner(area);
    f.render_widget(outer, area);

    let mut constraints = vec![Constraint::Length(2)]; // title
    for _ in &items {
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Length(1)); // footer

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
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

    let footer_idx = items.len() + 1;
    for (i, item) in items.iter().enumerate() {
        draw_item(f, chunks[i + 1], item, theme);
    }

    f.render_widget(
        Paragraph::new(Line::from(Span::styled(
            "q / Esc  quit",
            Style::default().fg(theme.footer),
        ))),
        chunks[footer_idx],
    );
}

fn draw_item(f: &mut ratatui::Frame, area: Rect, item: &ProgressItem, theme: &Theme) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

    let pct_str = format!("{:.1}%", item.fraction * 100.0);
    let label_str = format!("{:<6}", item.label);
    let detail_w = item.detail.chars().count();
    let pad = (area.width as usize).saturating_sub(label_str.len() + detail_w + pct_str.len());

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
        spans.push(Span::styled(
            FILLED.repeat(filled),
            Style::default().fg(item.color),
        ));
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
