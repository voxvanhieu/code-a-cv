//! The one date formatter, in Rust. Both renderers print the strings this
//! module produces: the PDF view carries them per locale, the HTML asks the
//! same functions. Locale matching uses the primary subtag, so `de-DE`
//! formats as `de`. The English spellings are the 0.3 display forms; `de`,
//! `fr`, `vi`, and `ja` follow the locale's order (day-first, or year-first
//! for Japanese). Japanese era dates (和暦) live here too, so no renderer or
//! theme keeps a second era table.

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

/// Match a locale tag by its primary subtag: `de-DE` formats as `de`.
fn primary_locale(locale: &str) -> &'static str {
    let base = locale
        .split(['-', '_'])
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match base.as_str() {
        "de" => "de",
        "fr" => "fr",
        "vi" => "vi",
        "ja" => "ja",
        _ => "en",
    }
}

/// The Japanese era for a date, compared by (year, month, day) against the
/// official era starts. Year 1 renders as 元年.
fn era_of(date: chrono::NaiveDate) -> Option<(&'static str, i32)> {
    let year = date.year();
    const ERAS: [(i32, u32, u32, &str); 5] = [
        (1868, 10, 23, "明治"),
        (1912, 7, 30, "大正"),
        (1926, 12, 25, "昭和"),
        (1989, 1, 8, "平成"),
        (2019, 5, 1, "令和"),
    ];
    let mut active: Option<(&'static str, i32)> = None;
    for (start_year, start_month, start_day, name) in ERAS {
        let start = chrono::NaiveDate::from_ymd_opt(start_year, start_month, start_day)?;
        if date >= start {
            active = Some((name, year - start_year + 1));
        }
    }
    active
}

/// The era-year token: year 1 is 元, every other year its numeral. The 年
/// comes from the surrounding format so 元年 never doubles it.
fn era_year(year: i32) -> String {
    if year == 1 {
        "元".to_owned()
    } else {
        year.to_string()
    }
}

/// A Japanese-era spelling of a full date: 令和8年1月15日. Dates before the
/// Meiji era render as Gregorian 西暦.
pub fn era_date_full(date: chrono::NaiveDate) -> String {
    match era_of(date) {
        Some((name, year)) => format!(
            "{name}{}年{}月{}日",
            era_year(year),
            date.month(),
            date.day()
        ),
        None => format!("西暦{}年{}月{}日", date.year(), date.month(), date.day()),
    }
}

/// A Japanese-era spelling of a date point; `None` for points that carry no
/// day or month beyond the era-year granularity.
pub fn era_date_point(value: &DatePoint) -> Option<String> {
    match value {
        DatePoint::Full(date) => Some(era_date_full(*date)),
        DatePoint::YearMonth(year, month) => {
            let month = u32::from(*month);
            let date = chrono::NaiveDate::from_ymd_opt(*year, month, 1)?;
            let (name, era) = era_of(date)?;
            Some(format!("{name}{}年{month}月", era_year(era)))
        }
        DatePoint::Year(year) => {
            let date = chrono::NaiveDate::from_ymd_opt(*year, 1, 1)?;
            let (name, era) = era_of(date)?;
            Some(format!("{name}{}年", era_year(era)))
        }
        DatePoint::Present => Some("現在".to_owned()),
    }
}

/// A Japanese-era spelling of a period, for form themes with a 和暦 column.
pub fn era_period(period: &Period) -> Option<String> {
    let start = period.start.as_ref().and_then(era_date_point);
    let end = period.end.as_ref().and_then(era_date_point);
    match (start, end) {
        (Some(start), Some(end)) => Some(format!("{start} – {end}")),
        (Some(start), None) => Some(format!("{start} –")),
        (None, Some(end)) => Some(format!("– {end}")),
        (None, None) => None,
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
    let name = primary_locale(locale);
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
                // German writes the day with a period: 15. Januar 2023.
                "de" => format!("{}. {month} {}", date.day(), date.year()),
                "fr" | "vi" => join_date(
                    name,
                    &[&date.day().to_string(), month, &date.year().to_string()],
                ),
                _ => format!("{month} {}, {}", date.day(), date.year()),
            }
        }
        DatePoint::Present => present_word(name).to_string(),
    }
}

/// Format a period as `start – end`. An absent endpoint keeps its dash, so
/// an ongoing range reads `2022 –` instead of silently becoming a single
/// date.
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
        (Some(start), None) => Some(format!("{start} –")),
        (None, Some(end)) => Some(format!("– {end}")),
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
    fn periods_render_with_the_en_dash_and_keep_open_ranges_open() {
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
        assert_eq!(format_period_locale("en", &open), Some("2022 –".into()));
        assert_eq!(format_period_locale("de", &open), Some("2022 –".into()));
        // An all-empty period is invalid at construction, so None never
        // reaches the formatter through validated data.
        assert!(Period::partial(None, None).is_err());
    }

    #[test]
    fn locales_match_by_primary_subtag() {
        assert_eq!(
            format_date_locale("de-DE", &DatePoint::YearMonth(2023, 3)),
            "März 2023"
        );
        assert_eq!(format_date_locale("ja-JP", &DatePoint::Present), "現在");
    }

    #[test]
    fn german_full_dates_use_the_period_after_the_day() {
        assert_eq!(
            format_date_locale("de", &full(2023, 1, 15)),
            "15. Januar 2023"
        );
    }

    #[test]
    fn era_dates_respect_the_month_boundaries_and_gannen() {
        use chrono::NaiveDate;
        let d = |y, m, day| NaiveDate::from_ymd_opt(y, m, day).unwrap();
        // The boundary: April 30 2019 is still 平成31, May 1 is 令和元年.
        assert_eq!(era_date_full(d(2019, 4, 30)), "平成31年4月30日");
        assert_eq!(era_date_full(d(2019, 5, 1)), "令和元年5月1日");
        assert_eq!(era_date_full(d(2026, 1, 15)), "令和8年1月15日");
        // Earlier eras and the switch days.
        assert_eq!(era_date_full(d(1989, 1, 7)), "昭和64年1月7日");
        assert_eq!(era_date_full(d(1989, 1, 8)), "平成元年1月8日");
        assert_eq!(era_date_full(d(1926, 12, 24)), "大正15年12月24日");
        assert_eq!(era_date_full(d(1926, 12, 25)), "昭和元年12月25日");
        // Before Meiji: Gregorian, spelled 西暦, so no column mixes eras.
        assert_eq!(era_date_full(d(1867, 12, 31)), "西暦1867年12月31日");
        // Granular points.
        assert_eq!(
            era_date_point(&DatePoint::YearMonth(2023, 8)),
            Some("令和5年8月".into())
        );
        // A year-only point has no month, so the era reads at the year's
        // start: 2019 begins inside 平成31.
        assert_eq!(
            era_date_point(&DatePoint::Year(2019)),
            Some("平成31年".into())
        );
        assert_eq!(
            era_date_point(&DatePoint::Year(2020)),
            Some("令和2年".into())
        );
        assert_eq!(era_date_point(&DatePoint::Present), Some("現在".into()));
        let period = Period::partial(
            Some(DatePoint::YearMonth(2023, 1)),
            Some(DatePoint::Present),
        )
        .unwrap();
        assert_eq!(era_period(&period), Some("令和5年1月 – 現在".into()));
    }
}
