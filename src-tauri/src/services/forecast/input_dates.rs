use chrono::{Duration, Months, NaiveDate, NaiveDateTime};

pub fn build_future_dates(last_date: &str, frequency: &str, horizon: u32) -> Vec<String> {
    let Some(last_datetime) = parse_datetime(last_date) else {
        return (1..=horizon).map(|index| format!("T+{index}")).collect();
    };
    let normalized = frequency.trim().to_uppercase();

    (1..=horizon)
        .map(|step| {
            shift_datetime(last_datetime, &normalized, step)
                .map(|value| format_output(value, last_date))
                .unwrap_or_else(|| format!("T+{step}"))
        })
        .collect()
}

fn parse_datetime(value: &str) -> Option<NaiveDateTime> {
    let formats = [
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M",
    ];
    for format in formats {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Some(parsed);
        }
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .or_else(|_| NaiveDate::parse_from_str(value, "%Y/%m/%d"))
        .ok()
        .and_then(|date| date.and_hms_opt(0, 0, 0))
}

fn shift_datetime(base: NaiveDateTime, frequency: &str, step: u32) -> Option<NaiveDateTime> {
    match frequency {
        "S" => Some(base + Duration::seconds(i64::from(step))),
        "T" | "MIN" => Some(base + Duration::minutes(i64::from(step))),
        "H" => Some(base + Duration::hours(i64::from(step))),
        "D" | "B" => Some(base + Duration::days(i64::from(step))),
        "W" => Some(base + Duration::weeks(i64::from(step))),
        "M" => base.checked_add_months(Months::new(step)),
        "Q" => base.checked_add_months(Months::new(step.saturating_mul(3))),
        "Y" | "A" => base.checked_add_months(Months::new(step.saturating_mul(12))),
        _ => parse_compound_frequency(base, frequency, step),
    }
}

fn parse_compound_frequency(
    base: NaiveDateTime,
    frequency: &str,
    step: u32,
) -> Option<NaiveDateTime> {
    let digits_len = frequency
        .chars()
        .take_while(|char| char.is_ascii_digit())
        .count();
    if digits_len == 0 || digits_len >= frequency.len() {
        return None;
    }
    let factor = frequency[..digits_len].parse::<u32>().ok()?;
    let unit = &frequency[digits_len..];
    let total = factor.saturating_mul(step);
    match unit {
        "S" => Some(base + Duration::seconds(i64::from(total))),
        "MIN" | "T" => Some(base + Duration::minutes(i64::from(total))),
        "H" => Some(base + Duration::hours(i64::from(total))),
        "D" => Some(base + Duration::days(i64::from(total))),
        "W" => Some(base + Duration::weeks(i64::from(total))),
        _ => None,
    }
}

fn format_output(value: NaiveDateTime, source: &str) -> String {
    if source.contains('T') {
        return value.format("%Y-%m-%dT%H:%M:%S").to_string();
    }
    if source.contains(':') {
        return value.format("%Y-%m-%d %H:%M:%S").to_string();
    }
    value.date().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
#[path = "input_dates_tests.rs"]
mod tests;
