// SPDX-License-Identifier: GPL-3.0-or-later
// Ported from Hurmet (https://hurmet.org), Copyright (c) 2026 Ronald Kok, MIT License; see LICENSE-HURMET.

//! Date literals, `'yyyy-mm-dd'` (date.js): their RPN (`⌾seconds`) and their TeX display.

use regex::Regex;
use std::sync::LazyLock;

static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^'(\d{4})-(\d{1,2})-(\d{1,2})'$").unwrap());

static DATE_FORMAT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(?:(w{3,4}), )?(y{1,4}|m{3,4}|d{1,2})([- .]| de )(m{2,4}|d{1,2})([- .]|, | de )(y{4}|d{2})(?: ([a-z]{2,3}))?$").unwrap()
});

/// Days since 1970-01-01 for a proleptic Gregorian date (H. Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 => 29,
        _ => 28,
    }
}

/// Seconds after 1970-01-01 UTC, or `None` for an invalid date (JavaScript's `NaN`).
pub fn seconds_from_iso_string(date_str: &str) -> Option<i64> {
    let c = DATE_RE.captures(date_str)?;
    let y: i64 = c[1].parse().ok()?;
    let m: i64 = c[2].parse().ok()?;
    let d: i64 = c[3].parse().ok()?;
    if !(1..=12).contains(&m) || d < 1 || d > days_in_month(y, m) {
        return None;
    }
    Some(days_from_civil(y, m, d) * 86_400)
}

/// parser.js uses date.js `dateRPN`.
pub(crate) fn date_rpn(date_str: &str) -> String {
    match seconds_from_iso_string(date_str) {
        Some(s) => format!("⌾{s}"),
        None => "⌾NaN".to_string(),
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

fn process_dmy(dmy: &str, (y, m, d): (i64, i64, i64)) -> String {
    match dmy {
        "d" => d.to_string(),
        "dd" => format!("{d:02}"),
        "mm" => format!("{m:02}"),
        // Hurmet uses Intl.DateTimeFormat here; only English month names are ported.
        "mmm" | "mmmm" => MONTHS[(m - 1) as usize][..3].to_string(),
        _ => y.to_string(),
    }
}

/// TeX for a date literal inside an expression (`dateDisplayFromIsoString(str, fmt, true)`).
pub(crate) fn date_tex(date_str: &str, date_format: Option<&str>) -> String {
    let Some(secs) = seconds_from_iso_string(date_str) else {
        return "\\text{Invalid Date}".to_string();
    };
    let ymd = civil_from_days(secs.div_euclid(86_400));
    let plain = match date_format {
        Some(f) if f != "yyyy-mm-dd" && !f.contains("de") => match DATE_FORMAT_RE.captures(f) {
            Some(c) => format!(
                "{}{}{}{}{}",
                process_dmy(&c[2], ymd),
                &c[3],
                process_dmy(&c[4], ymd),
                &c[5],
                process_dmy(&c[6], ymd)
            ),
            None => iso(ymd),
        },
        _ => iso(ymd),
    };
    format!("\\text{{{plain}}}")
}

fn iso((y, m, d): (i64, i64, i64)) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpn() {
        assert_eq!(date_rpn("'2025-01-20'"), "⌾1737331200");
        assert_eq!(date_rpn("'2000-1-1'"), "⌾946684800");
        assert_eq!(
            date_tex("'2025-4-18'", Some("yyy-mm-dd")),
            "\\text{2025-04-18}"
        );
    }
}
