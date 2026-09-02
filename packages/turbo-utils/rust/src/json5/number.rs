fn parse_number(token: &str, start: usize) -> Result<Number, Json5Error> {
    let (negative, unsigned) = if let Some(value) = token.strip_prefix('-') {
        (true, value)
    } else if let Some(value) = token.strip_prefix('+') {
        (false, value)
    } else {
        (false, token)
    };
    if unsigned.is_empty() {
        return Err(Json5Error::InvalidNumber(start));
    }

    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(Json5Error::InvalidNumber(start));
        }
        let value = u64::from_str_radix(hex, 16).map_err(|_| Json5Error::InvalidNumber(start))?;
        if negative {
            let magnitude = i128::from(value);
            let signed = -magnitude;
            if let Ok(value) = i64::try_from(signed) {
                return Ok(Number::from(value));
            }
            let value = format!("-{value}")
                .parse::<f64>()
                .map_err(|_| Json5Error::InvalidNumber(start))?;
            return Number::from_f64(value).ok_or(Json5Error::InvalidNumber(start));
        }
        return Ok(Number::from(value));
    }

    if !unsigned.contains('.') && !unsigned.contains('e') && !unsigned.contains('E') {
        if negative {
            if let Ok(value) = token.parse::<i64>() {
                return Ok(Number::from(value));
            }
        } else if let Ok(value) = unsigned.parse::<u64>() {
            return Ok(Number::from(value));
        }
    }

    let mut normalized = String::with_capacity(token.len() + 2);
    if negative {
        normalized.push('-');
    }
    if unsigned.starts_with('.') {
        normalized.push('0');
    }
    normalized.push_str(unsigned);
    if unsigned.ends_with('.') {
        normalized.push('0');
    }
    let value = normalized
        .parse::<f64>()
        .map_err(|_| Json5Error::InvalidNumber(start))?;
    Number::from_f64(value).ok_or(Json5Error::InvalidNumber(start))
}
