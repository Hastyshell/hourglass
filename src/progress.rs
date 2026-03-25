use chrono::{Datelike, Local, Timelike};
use ratatui::style::Color;

use crate::theme::Theme;

pub struct ProgressItem {
    pub label:     &'static str,
    pub fraction:  f64,
    pub detail:    String,
    pub color:     Color,
    pub dim_color: Color,
}

pub fn get_progress_items(theme: &Theme) -> Vec<ProgressItem> {
    let now    = Local::now();
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let hour   = now.hour()   as f64;

    let hour_frac = (minute * 60.0 + second) / 3600.0;

    let day_secs  = hour * 3600.0 + minute * 60.0 + second;
    let day_frac  = day_secs / 86400.0;

    let weekday   = now.weekday().num_days_from_monday() as f64;
    let week_frac = (weekday * 86400.0 + day_secs) / (7.0 * 86400.0);

    let days_in_month = days_in_current_month(&now);
    let month_frac    = ((now.day() as f64 - 1.0) * 86400.0 + day_secs)
        / (days_in_month as f64 * 86400.0);

    let days_in_year = if now.naive_local().date().leap_year() { 366.0 } else { 365.0 };
    let year_frac    = ((now.ordinal() as f64 - 1.0) * 86400.0 + day_secs)
        / (days_in_year * 86400.0);

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
