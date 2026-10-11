pub fn validate_date_ymd(date_str: &str) -> Result<(), String> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() != 3 {
        return Err("Formato de data inválido. Deve ser YYYY-MM-DD".to_string());
    }

    let y: u32 = parts[0].parse().map_err(|_| "Ano inválido".to_string())?;
    let m: u32 = parts[1].parse().map_err(|_| "Mês inválido".to_string())?;
    let d: u32 = parts[2].parse().map_err(|_| "Dia inválido".to_string())?;

    if !(1970..=2200).contains(&y) || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return Err("Data fora dos limites suportados".to_string());
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_date_ymd() {
        assert!(validate_date_ymd("2026-10-15").is_ok());
        assert!(validate_date_ymd("invalid-date").is_err());
        assert!(validate_date_ymd("2026-13-01").is_err());
        assert!(validate_date_ymd("2026-00-01").is_err());
    }

    #[test]
    fn test_format_year_month() {
        assert_eq!(format_year_month(2026, 10).unwrap(), "2026-10");
        assert!(format_year_month(2026, 13).is_err());
        assert!(format_year_month(1800, 5).is_err());
    }
}
