//! The shared date formatter: one locale-aware spelling for both
//! renderers, matching the contract's month tables
//! (`crates/cac-render/src/typst/contract.typ`). The English spellings are
//! the 0.3 display forms, so the deterministic PDF output stays
//! byte-identical; `de`, `fr`, `vi`, and `ja` follow the locale's order
//! (day-first, or year-first for Japanese).

use cac_core::{DatePoint, Period};
use chrono::Datelike;

const MONTHS_EN: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
const MONTHS_DE: [&str; 12] = [
    "Januar",
    "Februar",
    "März",
    "April",
    "Mai",
    "Juni",
    "Juli",
    "August",
    "September",
    "Oktober",
    "November",
    "Dezember",
];
const MONTHS_FR: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];
const MONTHS_VI: [&str; 12] = [
    "Tháng Một",
    "Tháng Hai",
    "Tháng Ba",
    "Tháng Tư",
    "Tháng Năm",
    "Tháng Sáu",
    "Tháng Bảy",
    "Tháng Tám",
    "Tháng Chín",
    "Tháng Mười",
    "Tháng Mười Một",
    "Tháng Mười Hai",
];
const MONTHS_JA: [&str; 12] = [
    "1月", "2月", "3月", "4月", "5月", "6月", "7月", "8月", "9月", "10月", "11月", "12月",
];

fn months(locale: &str) -> &'static [&'static str; 12] {
    match locale {
        "de" => &MONTHS_DE,
        "fr" => &MONTHS_FR,
        "vi" => &MONTHS_VI,
        "ja" => &MONTHS_JA,
        _ => &MONTHS_EN,
    }
}

fn present_word(locale: &str) -> &'static str {
    match locale {
        "de" => "Heute",
        "fr" => "Aujourd'hui",
        "vi" => "Nay",
        "ja" => "現在",
        _ => "Present",
    }
}

fn join_date(_locale: &str, parts: &[&str]) -> String {
    parts.join(" ")
}

/// Format one date point in the locale's order. English keeps the 0.3
/// spellings (`2023`, `Jan 2023`, `Jan 5, 2023`, `Present`).
pub fn format_date_locale(locale: &str, value: &DatePoint) -> String {
    let name = if matches!(locale, "de" | "fr" | "vi" | "ja" | "en") {
        locale
    } else {
        "en"
    };
    match value {
        DatePoint::Year(year) => year.to_string(),
        DatePoint::YearMonth(year, month) => {
            let name_of_month = months(name)
                .get(usize::from(*month).saturating_sub(1))
                .copied()
                .unwrap_or("???");
            match name {
                "ja" => format!("{year}年{name_of_month}"),
                "de" | "fr" | "vi" => format!("{name_of_month} {year}"),
                _ => format!("{name_of_month} {year}"),
            }
        }
        DatePoint::Full(date) => {
            let month = months(name)
                .get(usize::try_from(date.month()).unwrap_or(1).saturating_sub(1))
                .copied()
                .unwrap_or("???");
            match name {
                "ja" => format!("{}年{}月{}日", date.year(), date.month(), date.day()),
                "de" | "fr" | "vi" => join_date(
                    name,
                    &[&date.day().to_string(), month, &date.year().to_string()],
                ),
                _ => format!("{month} {}, {}", date.day(), date.year()),
            }
        }
        DatePoint::Present => present_word(name).to_string(),
    }
}

/// Format a period as `start – end`, omitting absent endpoints; `None` when
/// both are absent.
pub fn format_period_locale(locale: &str, period: &Period) -> Option<String> {
    let start = period
        .start
        .as_ref()
        .map(|value| format_date_locale(locale, value));
    let end = period
        .end
        .as_ref()
        .map(|value| format_date_locale(locale, value));
    match (start, end) {
        (Some(start), Some(end)) => Some(format!("{start} – {end}")),
        (Some(start), None) => Some(start),
        (None, Some(end)) => Some(end),
        (None, None) => None,
    }
}

/// A single date or period, whichever the entry carries, in the locale.
pub fn format_event_locale(
    locale: &str,
    period: Option<&Period>,
    date: Option<&DatePoint>,
) -> Option<String> {
    period
        .and_then(|value| format_period_locale(locale, value))
        .or_else(|| date.map(|value| format_date_locale(locale, value)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn full(y: i32, m: u32, d: u32) -> DatePoint {
        DatePoint::Full(NaiveDate::from_ymd_opt(y, m, d).unwrap())
    }

    #[test]
    fn english_keeps_the_0_3_spellings() {
        assert_eq!(format_date_locale("en", &DatePoint::Year(2023)), "2023");
        assert_eq!(
            format_date_locale("en", &DatePoint::YearMonth(2023, 1)),
            "Jan 2023"
        );
        assert_eq!(format_date_locale("en", &full(2023, 1, 5)), "Jan 5, 2023");
        assert_eq!(format_date_locale("en", &DatePoint::Present), "Present");
        assert_eq!(
            format_date_locale("unknown", &DatePoint::Present),
            "Present"
        );
    }

    #[test]
    fn locale_orders_follow_the_contract() {
        assert_eq!(
            format_date_locale("de", &DatePoint::YearMonth(2023, 3)),
            "März 2023"
        );
        assert_eq!(
            format_date_locale("ja", &DatePoint::YearMonth(2023, 8)),
            "2023年8月"
        );
        assert_eq!(
            format_date_locale("vi", &DatePoint::YearMonth(2022, 10)),
            "Tháng Mười 2022"
        );
        assert_eq!(format_date_locale("ja", &DatePoint::Present), "現在");
        assert_eq!(
            format_date_locale("ja", &full(2023, 8, 15)),
            "2023年8月15日"
        );
    }

    #[test]
    fn periods_render_with_the_en_dash_and_omit_absent_sides() {
        let period = Period::partial(
            Some(DatePoint::YearMonth(2023, 1)),
            Some(DatePoint::Present),
        )
        .unwrap();
        assert_eq!(
            format_period_locale("en", &period),
            Some("Jan 2023 – Present".into())
        );
        let open = Period::partial(Some(DatePoint::Year(2022)), None).unwrap();
        assert_eq!(format_period_locale("en", &open), Some("2022".into()));
        // An all-empty period is invalid at construction, so None never
        // reaches the formatter through validated data.
        assert!(Period::partial(None, None).is_err());
    }
}
