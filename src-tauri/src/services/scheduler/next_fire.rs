use crate::models::WakeupSchedule;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDateTime, NaiveTime, TimeZone, Weekday};

/// Renvoie la prochaine date/heure de déclenchement strictement future.
/// `None` si :
/// - Once dont la date est passée
/// - parsing échoue
pub fn next_fire_at(schedule: &WakeupSchedule, now: DateTime<Local>) -> Option<DateTime<Local>> {
    match schedule {
        WakeupSchedule::Once { datetime } => parse_once(datetime, now),
        WakeupSchedule::Daily { time } => parse_daily(time, now),
        WakeupSchedule::Weekly { weekday, time } => parse_weekly(*weekday, time, now),
    }
}

fn parse_once(s: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok()?;
    let dt = Local.from_local_datetime(&naive).single()?;
    if dt > now { Some(dt) } else { None }
}

fn parse_time_hhmm(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, "%H:%M").ok()
}

fn parse_daily(time: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let nt = parse_time_hhmm(time)?;
    let today_at = Local
        .from_local_datetime(&now.date_naive().and_time(nt))
        .single()?;
    if today_at > now {
        Some(today_at)
    } else {
        let tomorrow = (now.date_naive() + Duration::days(1)).and_time(nt);
        Local.from_local_datetime(&tomorrow).single()
    }
}

fn parse_weekly(weekday_idx: u8, time: &str, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let target_wd = weekday_from_idx(weekday_idx)?;
    let nt = parse_time_hhmm(time)?;

    let today_idx = now.weekday().num_days_from_monday() as i64;
    let target_idx = target_wd.num_days_from_monday() as i64;
    let mut delta = target_idx - today_idx;
    if delta < 0 {
        delta += 7;
    }

    let candidate = (now.date_naive() + Duration::days(delta)).and_time(nt);
    let dt = Local.from_local_datetime(&candidate).single()?;
    if dt > now {
        Some(dt)
    } else {
        // Même jour mais heure passée → semaine suivante
        let next_week = (now.date_naive() + Duration::days(delta + 7)).and_time(nt);
        Local.from_local_datetime(&next_week).single()
    }
}

fn weekday_from_idx(idx: u8) -> Option<Weekday> {
    match idx {
        0 => Some(Weekday::Mon),
        1 => Some(Weekday::Tue),
        2 => Some(Weekday::Wed),
        3 => Some(Weekday::Thu),
        4 => Some(Weekday::Fri),
        5 => Some(Weekday::Sat),
        6 => Some(Weekday::Sun),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn local(y: i32, m: u32, d: u32, h: u32, mi: u32) -> DateTime<Local> {
        Local
            .with_ymd_and_hms(y, m, d, h, mi, 0)
            .single()
            .expect("valid datetime")
    }

    #[test]
    fn once_future_ok() {
        let now = local(2026, 4, 16, 8, 0);
        let s = WakeupSchedule::Once { datetime: "2026-04-16T08:30".into() };
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 30)));
    }

    #[test]
    fn once_past_none() {
        let now = local(2026, 4, 16, 9, 0);
        let s = WakeupSchedule::Once { datetime: "2026-04-16T08:30".into() };
        assert_eq!(next_fire_at(&s, now), None);
    }

    #[test]
    fn daily_today_if_future() {
        let now = local(2026, 4, 16, 7, 0);
        let s = WakeupSchedule::Daily { time: "08:00".into() };
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 0)));
    }

    #[test]
    fn daily_tomorrow_if_past() {
        let now = local(2026, 4, 16, 9, 0);
        let s = WakeupSchedule::Daily { time: "08:00".into() };
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 17, 8, 0)));
    }

    #[test]
    fn weekly_same_day_future() {
        // 2026-04-16 = jeudi (idx 3)
        let now = local(2026, 4, 16, 7, 0);
        let s = WakeupSchedule::Weekly { weekday: 3, time: "08:00".into() };
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 16, 8, 0)));
    }

    #[test]
    fn weekly_next_week_if_passed() {
        let now = local(2026, 4, 16, 9, 0);
        let s = WakeupSchedule::Weekly { weekday: 3, time: "08:00".into() };
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 23, 8, 0)));
    }

    #[test]
    fn weekly_other_day() {
        let now = local(2026, 4, 16, 9, 0); // jeudi
        let s = WakeupSchedule::Weekly { weekday: 0, time: "10:00".into() }; // lundi
        // prochain lundi = 20 avril
        assert_eq!(next_fire_at(&s, now), Some(local(2026, 4, 20, 10, 0)));
    }
}
