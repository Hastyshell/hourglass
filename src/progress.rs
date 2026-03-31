use chrono::{Datelike, Local, NaiveDate, NaiveTime, Timelike};
use ratatui::style::Color;

use crate::theme::Theme;

pub struct ProgressItem {
    pub label: &'static str,
    pub fraction: f64,
    pub detail: String,
    pub color: Color,
    pub dim_color: Color,
}

pub fn get_progress_items(
    theme: &Theme,
    birth: Option<NaiveDate>,
    lifespan: u32,
    day_start: NaiveTime,
    day_end: NaiveTime,
) -> Vec<ProgressItem> {
    let now = Local::now();
    let minute = now.minute() as f64;
    let second = now.second() as f64;
    let hour = now.hour() as f64;

    let hour_frac = (minute * 60.0 + second) / 3600.0;

    let day_secs = hour * 3600.0 + minute * 60.0 + second;
    let day_frac = day_fraction(now.time(), day_start, day_end);

    let weekday = now.weekday().num_days_from_monday() as f64;
    let week_frac = (weekday * 86400.0 + day_secs) / (7.0 * 86400.0);

    let days_in_month = days_in_current_month(&now);
    let month_frac =
        ((now.day() as f64 - 1.0) * 86400.0 + day_secs) / (days_in_month as f64 * 86400.0);

    let days_in_year = if now.naive_local().date().leap_year() {
        366.0
    } else {
        365.0
    };
    let year_frac = ((now.ordinal() as f64 - 1.0) * 86400.0 + day_secs) / (days_in_year * 86400.0);

    let wday = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let month = [
        "", "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let fracs = [hour_frac, day_frac, week_frac, month_frac, year_frac];
    let labels = ["Hour", "Day", "Week", "Month", "Year"];
    let details = [
        format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second()),
        format!(
            "{} {:02}:{:02}",
            wday[now.weekday().num_days_from_monday() as usize],
            now.hour(),
            now.minute()
        ),
        format!(
            "{}  {} ∕ 7",
            wday[now.weekday().num_days_from_monday() as usize],
            now.weekday().num_days_from_monday() + 1
        ),
        format!(
            "{}  {} ∕ {}",
            month[now.month() as usize],
            now.day(),
            days_in_month
        ),
        format!("{}  {} ∕ {:.0}", now.year(), now.ordinal(), days_in_year),
    ];

    let mut items: Vec<ProgressItem> = labels
        .iter()
        .enumerate()
        .map(|(i, &label)| ProgressItem {
            label,
            fraction: fracs[i],
            detail: details[i].clone(),
            color: theme.items[i].0,
            dim_color: theme.items[i].1,
        })
        .collect();

    if let Some(b) = birth {
        let today = now.date_naive();
        let days_lived = (today - b).num_days().max(0) as f64;
        let total_days = lifespan as f64 * 365.25;
        let life_frac = (days_lived / total_days).clamp(0.0, 1.0);
        let age = (days_lived / 365.25).floor() as u32;
        items.push(ProgressItem {
            label: "Life",
            fraction: life_frac,
            detail: format!("{}  {} ∕ {}", b.year(), age, lifespan),
            color: theme.items[5].0,
            dim_color: theme.items[5].1,
        });
    }

    items
}

fn day_fraction(now: NaiveTime, day_start: NaiveTime, day_end: NaiveTime) -> f64 {
    let now_secs = now.num_seconds_from_midnight();
    let start_secs = day_start.num_seconds_from_midnight();
    let end_secs = day_end.num_seconds_from_midnight();

    if start_secs == end_secs {
        return now_secs as f64 / 86_400.0;
    }

    if start_secs < end_secs {
        let total = end_secs - start_secs;
        let elapsed = if now_secs <= start_secs {
            0
        } else if now_secs >= end_secs {
            total
        } else {
            now_secs - start_secs
        };
        return elapsed as f64 / total as f64;
    }

    let total = 86_400 - start_secs + end_secs;
    let elapsed = if now_secs >= start_secs {
        now_secs - start_secs
    } else if now_secs <= end_secs {
        86_400 - start_secs + now_secs
    } else {
        0
    };

    elapsed as f64 / total as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    // --- days_in_current_month ---

    #[test]
    fn days_in_december() {
        let dt = Local.with_ymd_and_hms(2024, 12, 15, 0, 0, 0).unwrap();
        assert_eq!(days_in_current_month(&dt), 31);
    }

    #[test]
    fn days_in_february_leap_year() {
        let dt = Local.with_ymd_and_hms(2024, 2, 1, 0, 0, 0).unwrap();
        assert_eq!(days_in_current_month(&dt), 29);
    }

    #[test]
    fn days_in_february_non_leap_year() {
        let dt = Local.with_ymd_and_hms(2023, 2, 1, 0, 0, 0).unwrap();
        assert_eq!(days_in_current_month(&dt), 28);
    }

    // --- life fraction ---

    fn life_frac(today: NaiveDate, birth: NaiveDate, lifespan: u32) -> f64 {
        let days_lived = (today - birth).num_days().max(0) as f64;
        let total_days = lifespan as f64 * 365.25;
        (days_lived / total_days).clamp(0.0, 1.0)
    }

    #[test]
    fn life_frac_future_birth_clamps_to_zero() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let birth = NaiveDate::from_ymd_opt(2030, 6, 1).unwrap();
        assert_eq!(life_frac(today, birth, 80), 0.0);
    }

    #[test]
    fn life_frac_exceeds_lifespan_clamps_to_one() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let birth = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        assert_eq!(life_frac(today, birth, 80), 1.0);
    }

    #[test]
    fn life_frac_midpoint() {
        let today = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let birth = NaiveDate::from_ymd_opt(1986, 1, 1).unwrap();
        let f = life_frac(today, birth, 80);
        assert!((f - 0.5).abs() < 0.01, "expected ~0.5, got {f}");
    }

    // --- day fraction ---

    #[test]
    fn day_frac_defaults_to_full_day() {
        let now = NaiveTime::from_hms_opt(12, 0, 0).unwrap();
        let midnight = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        assert_eq!(day_fraction(now, midnight, midnight), 0.5);
    }

    #[test]
    fn day_frac_before_start_clamps_to_zero() {
        let now = NaiveTime::from_hms_opt(7, 30, 0).unwrap();
        let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        assert_eq!(day_fraction(now, start, end), 0.0);
    }

    #[test]
    fn day_frac_after_end_clamps_to_one() {
        let now = NaiveTime::from_hms_opt(23, 30, 0).unwrap();
        let start = NaiveTime::from_hms_opt(8, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(23, 0, 0).unwrap();
        assert_eq!(day_fraction(now, start, end), 1.0);
    }

    #[test]
    fn day_frac_inside_custom_window() {
        let now = NaiveTime::from_hms_opt(15, 30, 0).unwrap();
        let start = NaiveTime::from_hms_opt(9, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(21, 0, 0).unwrap();
        let f = day_fraction(now, start, end);
        assert!(
            (f - 0.541_666_666_7).abs() < 1e-9,
            "expected ~0.5417, got {f}"
        );
    }

    #[test]
    fn day_frac_wraps_past_midnight() {
        let now = NaiveTime::from_hms_opt(1, 0, 0).unwrap();
        let start = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
        let f = day_fraction(now, start, end);
        assert!((f - 0.875).abs() < 1e-9, "expected 0.875, got {f}");
    }

    #[test]
    fn day_frac_wrap_gap_clamps_to_zero() {
        let now = NaiveTime::from_hms_opt(10, 0, 0).unwrap();
        let start = NaiveTime::from_hms_opt(18, 0, 0).unwrap();
        let end = NaiveTime::from_hms_opt(2, 0, 0).unwrap();
        assert_eq!(day_fraction(now, start, end), 0.0);
    }
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
