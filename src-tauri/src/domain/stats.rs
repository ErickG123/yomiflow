use crate::models::StreakStats;

pub fn validate_date_ymd(date_str: &str) -> Result<(), String> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err("Formato de data inválido. Deve ser YYYY-MM-DD".to_string());
    }

    let y: u32 = parts[0].parse().map_err(|_| "Ano inválido".to_string())?;
    let m: u32 = parts[1].parse().map_err(|_| "Mês inválido".to_string())?;
    let d: u32 = parts[2].parse().map_err(|_| "Dia inválido".to_string())?;

    if !(1970..=2200).contains(&y) || !(1..=12).contains(&m) {
        return Err("Data fora dos limites suportados".to_string());
    }
    let days_in_month = match m {
        2 if y.is_multiple_of(400) || (y.is_multiple_of(4) && !y.is_multiple_of(100)) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    if !(1..=days_in_month).contains(&d) {
        return Err("Dia inválido para o mês informado".to_string());
    }

    Ok(())
}

pub fn format_year_month(year: u32, month: u32) -> Result<String, String> {
    if !(1..=12).contains(&month) {
        return Err("Mês deve estar entre 1 e 12".to_string());
    }
    if !(1970..=2200).contains(&year) {
        return Err("Ano fora dos limites suportados".to_string());
    }
    Ok(format!("{:04}-{:02}", year, month))
}

/// Computes current and maximum consecutive review-day counts from local dates.
pub fn calculate_streaks(today: &str, review_dates: &[String]) -> StreakStats {
    let Some(today_day) = date_to_days(today) else {
        return StreakStats {
            current_streak: 0,
            max_streak: 0,
        };
    };

    let mut days = review_dates
        .iter()
        .filter_map(|date| date_to_days(date))
        .collect::<Vec<_>>();
    days.sort_unstable_by(|left, right| right.cmp(left));
    days.dedup();
    if days.is_empty() {
        return StreakStats {
            current_streak: 0,
            max_streak: 0,
        };
    }

    let mut current_streak = 0;
    let mut expected_day = if days[0] == today_day {
        Some(today_day)
    } else if days[0] == today_day - 1 {
        Some(today_day - 1)
    } else {
        None
    };
    for day in &days {
        if Some(*day) == expected_day {
            current_streak += 1;
            expected_day = expected_day.map(|expected| expected - 1);
        } else if expected_day.is_some_and(|expected| *day > expected) {
            continue;
        } else {
            break;
        }
    }

    let mut max_streak = 1;
    let mut current_run = 1;
    for pair in days.windows(2) {
        if pair[0] - pair[1] == 1 {
            current_run += 1;
            max_streak = max_streak.max(current_run);
        } else {
            current_run = 1;
        }
    }

    StreakStats {
        current_streak,
        max_streak: max_streak.max(current_streak),
    }
}

fn date_to_days(date: &str) -> Option<i64> {
    validate_date_ymd(date).ok()?;
    let mut parts = date.split('-');
    let year = parts.next()?.parse::<i64>().ok()?;
    let month = parts.next()?.parse::<i64>().ok()?;
    let day = parts.next()?.parse::<i64>().ok()?;
    let adjusted_month = (month + 9) % 12;
    let adjusted_year = year - adjusted_month / 10;
    Some(
        365 * adjusted_year + adjusted_year / 4 - adjusted_year / 100
            + adjusted_year / 400
            + (adjusted_month * 306 + 5) / 10
            + (day - 1),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_ymd() {
        assert!(validate_date_ymd("2026-10-15").is_ok());
        assert!(validate_date_ymd("invalid-date").is_err());
        assert!(validate_date_ymd("2026-13-01").is_err());
        assert!(validate_date_ymd("2026-00-01").is_err());
        assert!(validate_date_ymd("2026-02-29").is_err());
        assert!(validate_date_ymd("2024-02-29").is_ok());
        assert!(validate_date_ymd("2026-04-31").is_err());
    }

    #[test]
    fn test_format_year_month() {
        assert_eq!(format_year_month(2026, 10).unwrap(), "2026-10");
        assert!(format_year_month(2026, 13).is_err());
        assert!(format_year_month(1800, 5).is_err());
    }

    #[test]
    fn streak_calculation_is_independent_of_storage_and_order() {
        let dates = vec![
            "2026-05-03".to_owned(),
            "2026-05-01".to_owned(),
            "2026-05-02".to_owned(),
        ];
        let result = calculate_streaks("2026-05-03", &dates);
        assert_eq!(result.current_streak, 3);
        assert_eq!(result.max_streak, 3);
    }

    #[test]
    fn missing_today_keeps_yesterday_streak_but_breaks_max_streak_gaps() {
        let dates = vec!["2026-05-02".to_owned(), "2026-04-30".to_owned()];
        let result = calculate_streaks("2026-05-03", &dates);
        assert_eq!(result.current_streak, 1);
        assert_eq!(result.max_streak, 1);
    }
}
