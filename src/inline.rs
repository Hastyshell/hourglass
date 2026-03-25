use std::io::{self, Write};

use chrono::Local;
use crossterm::execute;
use crossterm::style::{Color as CColor, Print, ResetColor, SetForegroundColor};

use crate::progress::{ProgressItem, get_progress_items};
use crate::theme::{Theme, rgb, to_ct};
use crate::{EMPTY, FILLED, HEAD};

pub fn print_inline(theme: &Theme) -> io::Result<()> {
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
        print_item(&mut out, item, theme, width)?;
    }

    execute!(
        out,
        SetForegroundColor(CColor::Rgb { r: ft_r, g: ft_g, b: ft_b }),
        Print("-w / --watch        live updating mode\n--theme dark|light  color theme\n"),
        ResetColor,
    )?;

    out.flush()
}

fn print_item(
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
