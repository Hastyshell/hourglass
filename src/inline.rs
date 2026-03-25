use std::io::{self, Write};

use chrono::Local;
use crossterm::execute;
use crossterm::style::{Color as CColor, Print, ResetColor, SetForegroundColor};

use crate::progress::{ProgressItem, get_progress_items};
use crate::theme::{Theme, rgb, to_ct};
use crate::{EMPTY, FILLED, HEAD};

pub fn print_inline(theme: &Theme) -> io::Result<()> {
    let term_width  = crossterm::terminal::size().map(|(w, _)| w as usize).unwrap_or(80);
    let outer_width = term_width.min(72);
    // │ + 2-space padding + content + 2-space padding + │  =  outer_width
    let inner_width = outer_width.saturating_sub(6);

    let now   = Local::now();
    let items = get_progress_items(theme);
    let mut out = io::stdout();

    let bc               = to_ct(theme.tui_border);
    let (ta_r, ta_g, ta_b) = rgb(theme.title_accent);
    let (ts_r, ts_g, ts_b) = rgb(theme.timestamp);

    // ── Top border ────────────────────────────────────────────────────────────
    execute!(out,
        SetForegroundColor(bc),
        Print(format!("╭{}╮\n", "─".repeat(outer_width.saturating_sub(2)))),
        ResetColor,
    )?;

    // Empty top-padding line
    empty_line(&mut out, outer_width, bc)?;

    // ── Title ─────────────────────────────────────────────────────────────────
    // Visible width: ⏳(2) + " "(1) + "hourglass"(9) + "  "(2) + timestamp(19) = 33
    let ts_str = now.format("%Y-%m-%d %H:%M:%S").to_string();
    let title_visible = 2 + 1 + 9 + 2 + ts_str.len();
    let title_rpad    = inner_width.saturating_sub(title_visible);

    border_left(&mut out, bc)?;
    execute!(out,
        SetForegroundColor(CColor::Rgb { r: ta_r, g: ta_g, b: ta_b }),
        Print("⏳ hourglass"),
        SetForegroundColor(CColor::Rgb { r: ts_r, g: ts_g, b: ts_b }),
        Print(format!("  {}", ts_str)),
        Print(" ".repeat(title_rpad)),
    )?;
    border_right(&mut out, bc)?;

    // Blank separator line
    empty_line(&mut out, outer_width, bc)?;

    // ── Progress items ────────────────────────────────────────────────────────
    for item in &items {
        print_item(&mut out, item, theme, inner_width, outer_width, bc)?;
    }

    // ── Bottom border ─────────────────────────────────────────────────────────
    execute!(out,
        SetForegroundColor(bc),
        Print(format!("╰{}╯\n", "─".repeat(outer_width.saturating_sub(2)))),
        ResetColor,
    )?;

    out.flush()
}

fn print_item(
    out: &mut impl Write,
    item: &ProgressItem,
    theme: &Theme,
    inner_width: usize,
    outer_width: usize,
    bc: CColor,
) -> io::Result<()> {
    let pct_str   = format!("{:.1}%", item.fraction * 100.0);
    let label_str = format!("{:<6}", item.label);
    let detail_w  = item.detail.chars().count();
    let pad       = inner_width.saturating_sub(label_str.len() + detail_w + pct_str.len());
    let (dt_r, dt_g, dt_b) = rgb(theme.detail_text);

    // Label line
    border_left(out, bc)?;
    execute!(out,
        SetForegroundColor(to_ct(item.color)),
        Print(&label_str),
        SetForegroundColor(CColor::Rgb { r: dt_r, g: dt_g, b: dt_b }),
        Print(&item.detail),
        Print(" ".repeat(pad)),
        SetForegroundColor(to_ct(item.color)),
        Print(&pct_str),
        ResetColor,
    )?;
    border_right(out, bc)?;

    // Bar line
    let filled = ((inner_width as f64) * item.fraction).floor() as usize;
    let filled = filled.min(inner_width);

    border_left(out, bc)?;
    if filled > 0 {
        execute!(out, SetForegroundColor(to_ct(item.color)), Print(FILLED.repeat(filled)))?;
    }
    if filled < inner_width {
        execute!(out, SetForegroundColor(to_ct(item.color)), Print(HEAD))?;
        let remaining = inner_width.saturating_sub(filled + 1);
        if remaining > 0 {
            execute!(out, SetForegroundColor(to_ct(item.dim_color)), Print(EMPTY.repeat(remaining)))?;
        }
    }
    execute!(out, ResetColor)?;
    border_right(out, bc)?;

    // Spacer line
    empty_line(out, outer_width, bc)
}

// ── Border helpers ────────────────────────────────────────────────────────────

fn border_left(out: &mut impl Write, bc: CColor) -> io::Result<()> {
    execute!(out, SetForegroundColor(bc), Print("│"), ResetColor, Print("  "))
}

fn border_right(out: &mut impl Write, bc: CColor) -> io::Result<()> {
    execute!(out, SetForegroundColor(bc), Print("  │\n"), ResetColor)
}

fn empty_line(out: &mut impl Write, outer_width: usize, bc: CColor) -> io::Result<()> {
    execute!(out,
        SetForegroundColor(bc),
        Print("│"),
        ResetColor,
        Print(" ".repeat(outer_width.saturating_sub(2))),
        SetForegroundColor(bc),
        Print("│\n"),
        ResetColor,
    )
}
