pub fn parse_finite_number(raw: &str) -> Result<f64, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.chars().any(char::is_whitespace) {
        return Err("Valeur numérique invalide".into());
    }

    let normalized = normalize_separators(trimmed)?;
    let value = normalized
        .parse::<f64>()
        .map_err(|_| "Valeur numérique invalide".to_string())?;
    if !value.is_finite() {
        return Err("Valeur numérique invalide".into());
    }
    Ok(value)
}

fn normalize_separators(value: &str) -> Result<String, String> {
    let comma_count = value.matches(',').count();
    let dot_count = value.matches('.').count();
    match (comma_count, dot_count) {
        (0, 0) => Ok(value.to_string()),
        (0, _) => Ok(value.to_string()),
        (1, 0) => Ok(value.replace(',', ".")),
        _ => Err("Séparateur numérique ambigu".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_dot_or_single_decimal_comma() {
        assert_eq!(parse_finite_number("-12.5"), Ok(-12.5));
        assert_eq!(parse_finite_number("12,5"), Ok(12.5));
    }

    #[test]
    fn rejects_ambiguous_grouping() {
        assert!(parse_finite_number("1,234.56").is_err());
        assert!(parse_finite_number("1.234,56").is_err());
        assert!(parse_finite_number("1,234,56").is_err());
    }
}
