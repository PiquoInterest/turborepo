use thiserror::Error;

const MAX_NESTING_DEPTH: usize = 128;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Json5ScanError {
    #[error("expected a top-level object at byte {0}")]
    ExpectedObject(usize),
    #[error("expected an object key at byte {0}")]
    ExpectedKey(usize),
    #[error("expected ':' after object key at byte {0}")]
    ExpectedColon(usize),
    #[error("expected ',' or a closing delimiter at byte {0}")]
    ExpectedCommaOrEnd(usize),
    #[error("unexpected value at byte {0}")]
    UnexpectedValue(usize),
    #[error("invalid escape sequence at byte {0}")]
    InvalidEscape(usize),
    #[error("invalid unicode escape at byte {0}")]
    InvalidUnicodeEscape(usize),
    #[error("unterminated string at byte {0}")]
    UnterminatedString(usize),
    #[error("unterminated block comment at byte {0}")]
    UnterminatedComment(usize),
    #[error("unterminated object")]
    UnterminatedObject,
    #[error("unterminated array")]
    UnterminatedArray,
    #[error("JSON5 nesting exceeds 128 levels")]
    NestingTooDeep,
    #[error("unexpected trailing content at byte {0}")]
    TrailingContent(usize),
}

struct Scanner<'a> {
    input: &'a str,
    bytes: &'a [u8],
    index: usize,
}

impl<'a> Scanner<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            index: 0,
        }
    }

    fn skip_trivia(&mut self) -> Result<(), Json5ScanError> {
        loop {
            while self
                .bytes
                .get(self.index)
                .is_some_and(|value| value.is_ascii_whitespace())
            {
                self.index += 1;
            }

            if self.bytes.get(self.index) == Some(&b'/')
                && self.bytes.get(self.index + 1) == Some(&b'/')
            {
                self.index += 2;
                while self
                    .bytes
                    .get(self.index)
                    .is_some_and(|value| *value != b'\n' && *value != b'\r')
                {
                    self.index += 1;
                }
                continue;
            }

            if self.bytes.get(self.index) == Some(&b'/')
                && self.bytes.get(self.index + 1) == Some(&b'*')
            {
                let start = self.index;
                self.index += 2;
                let mut closed = false;
                while self.index + 1 < self.bytes.len() {
                    if self.bytes[self.index] == b'*' && self.bytes[self.index + 1] == b'/' {
                        self.index += 2;
                        closed = true;
                        break;
                    }
                    self.index += 1;
                }
                if !closed {
                    return Err(Json5ScanError::UnterminatedComment(start));
                }
                continue;
            }

            return Ok(());
        }
    }

    fn parse_hex_u16(&mut self, start: usize) -> Result<u16, Json5ScanError> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = *self
                .bytes
                .get(self.index)
                .ok_or(Json5ScanError::InvalidUnicodeEscape(start))?;
            let digit = match digit {
                b'0'..=b'9' => u16::from(digit - b'0'),
                b'a'..=b'f' => u16::from(digit - b'a') + 10,
                b'A'..=b'F' => u16::from(digit - b'A') + 10,
                _ => return Err(Json5ScanError::InvalidUnicodeEscape(start)),
            };
            value = value * 16 + digit;
            self.index += 1;
        }
        Ok(value)
    }

    fn parse_hex_byte(&mut self, start: usize) -> Result<u8, Json5ScanError> {
        let mut value = 0_u8;
        for _ in 0..2 {
            let digit = *self
                .bytes
                .get(self.index)
                .ok_or(Json5ScanError::InvalidEscape(start))?;
            let digit = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                b'A'..=b'F' => digit - b'A' + 10,
                _ => return Err(Json5ScanError::InvalidEscape(start)),
            };
            value = value * 16 + digit;
            self.index += 1;
        }
        Ok(value)
    }

    fn parse_unicode_escape(&mut self, start: usize) -> Result<char, Json5ScanError> {
        let first = self.parse_hex_u16(start)?;
        let code_point = if (0xD800..=0xDBFF).contains(&first) {
            if self.bytes.get(self.index) != Some(&b'\\')
                || self.bytes.get(self.index + 1) != Some(&b'u')
            {
                return Err(Json5ScanError::InvalidUnicodeEscape(start));
            }
            self.index += 2;
            let second = self.parse_hex_u16(start)?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(Json5ScanError::InvalidUnicodeEscape(start));
            }
            0x1_0000 + (u32::from(first - 0xD800) << 10) + u32::from(second - 0xDC00)
        } else if (0xDC00..=0xDFFF).contains(&first) {
            return Err(Json5ScanError::InvalidUnicodeEscape(start));
        } else {
            u32::from(first)
        };

        char::from_u32(code_point).ok_or(Json5ScanError::InvalidUnicodeEscape(start))
    }

    fn parse_string(&mut self) -> Result<String, Json5ScanError> {
        let start = self.index;
        let quote = *self
            .bytes
            .get(self.index)
            .ok_or(Json5ScanError::UnterminatedString(start))?;
        self.index += 1;
        let mut output = String::new();

        while let Some(value) = self.bytes.get(self.index).copied() {
            if value == quote {
                self.index += 1;
                return Ok(output);
            }
            if value < 0x20 {
                return Err(Json5ScanError::UnterminatedString(start));
            }
            if value != b'\\' {
                let remainder = self
                    .input
                    .get(self.index..)
                    .ok_or(Json5ScanError::UnterminatedString(start))?;
                let character = remainder
                    .chars()
                    .next()
                    .ok_or(Json5ScanError::UnterminatedString(start))?;
                output.push(character);
                self.index += character.len_utf8();
                continue;
            }

            let escape_start = self.index;
            self.index += 1;
            let escaped = *self
                .bytes
                .get(self.index)
                .ok_or(Json5ScanError::UnterminatedString(start))?;
            self.index += 1;
            match escaped {
                b'\'' => output.push('\''),
                b'"' => output.push('"'),
                b'\\' => output.push('\\'),
                b'/' => output.push('/'),
                b'b' => output.push('\u{0008}'),
                b'f' => output.push('\u{000c}'),
                b'n' => output.push('\n'),
                b'r' => output.push('\r'),
                b't' => output.push('\t'),
                b'v' => output.push('\u{000b}'),
                b'0' => {
                    if self
                        .bytes
                        .get(self.index)
                        .is_some_and(|value| value.is_ascii_digit())
                    {
                        return Err(Json5ScanError::InvalidEscape(escape_start));
                    }
                    output.push('\0');
                }
                b'x' => output.push(char::from(self.parse_hex_byte(escape_start)?)),
                b'u' => output.push(self.parse_unicode_escape(escape_start)?),
                b'\n' => {}
                b'\r' => {
                    if self.bytes.get(self.index) == Some(&b'\n') {
                        self.index += 1;
                    }
                }
                value if value.is_ascii() => output.push(char::from(value)),
                _ => return Err(Json5ScanError::InvalidEscape(escape_start)),
            }
        }

        Err(Json5ScanError::UnterminatedString(start))
    }

    fn parse_identifier(&mut self) -> Result<String, Json5ScanError> {
        let start = self.index;
        let first = self
            .bytes
            .get(self.index)
            .copied()
            .ok_or(Json5ScanError::ExpectedKey(start))?;
        if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b'$')) {
            return Err(Json5ScanError::ExpectedKey(start));
        }
        self.index += 1;
        while self.bytes.get(self.index).is_some_and(|value| {
            value.is_ascii_alphanumeric() || matches!(*value, b'_' | b'$')
        }) {
            self.index += 1;
        }
        Ok(self.input[start..self.index].to_owned())
    }

    fn parse_key(&mut self) -> Result<String, Json5ScanError> {
        match self.bytes.get(self.index).copied() {
            Some(b'\'' | b'"') => self.parse_string(),
            Some(value) if value.is_ascii_alphabetic() || matches!(value, b'_' | b'$') => {
                self.parse_identifier()
            }
            _ => Err(Json5ScanError::ExpectedKey(self.index)),
        }
    }

    fn parse_number_or_literal(&mut self) -> Result<(), Json5ScanError> {
        let start = self.index;
        while let Some(value) = self.bytes.get(self.index).copied() {
            if value.is_ascii_whitespace() || matches!(value, b',' | b']' | b'}') {
                break;
            }
            if value == b'/'
                && self
                    .bytes
                    .get(self.index + 1)
                    .is_some_and(|next| matches!(*next, b'/' | b'*'))
            {
                break;
            }
            if matches!(value, b'{' | b'[' | b':' | b'\'' | b'"') {
                return Err(Json5ScanError::UnexpectedValue(self.index));
            }
            self.index += 1;
        }

        let token = self
            .input
            .get(start..self.index)
            .ok_or(Json5ScanError::UnexpectedValue(start))?;
        let valid_literal = matches!(token, "true" | "false" | "null" | "Infinity" | "+Infinity" | "-Infinity" | "NaN" | "+NaN" | "-NaN");
        if valid_literal || valid_json5_number(token) {
            Ok(())
        } else {
            Err(Json5ScanError::UnexpectedValue(start))
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<(), Json5ScanError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(Json5ScanError::NestingTooDeep);
        }
        self.index += 1;
        self.skip_trivia()?;
        if self.bytes.get(self.index) == Some(&b']') {
            self.index += 1;
            return Ok(());
        }

        loop {
            self.parse_value(depth)?;
            self.skip_trivia()?;
            match self.bytes.get(self.index).copied() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_trivia()?;
                    if self.bytes.get(self.index) == Some(&b']') {
                        self.index += 1;
                        return Ok(());
                    }
                }
                Some(b']') => {
                    self.index += 1;
                    return Ok(());
                }
                None => return Err(Json5ScanError::UnterminatedArray),
                _ => return Err(Json5ScanError::ExpectedCommaOrEnd(self.index)),
            }
        }
    }

    fn parse_object(
        &mut self,
        depth: usize,
        collect_keys: bool,
        keys: &mut Vec<String>,
    ) -> Result<(), Json5ScanError> {
        if depth > MAX_NESTING_DEPTH {
            return Err(Json5ScanError::NestingTooDeep);
        }
        self.index += 1;
        self.skip_trivia()?;
        if self.bytes.get(self.index) == Some(&b'}') {
            self.index += 1;
            return Ok(());
        }

        loop {
            let key = self.parse_key()?;
            if collect_keys {
                keys.push(key);
            }
            self.skip_trivia()?;
            if self.bytes.get(self.index) != Some(&b':') {
                return Err(Json5ScanError::ExpectedColon(self.index));
            }
            self.index += 1;
            self.skip_trivia()?;
            self.parse_value(depth)?;
            self.skip_trivia()?;

            match self.bytes.get(self.index).copied() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_trivia()?;
                    if self.bytes.get(self.index) == Some(&b'}') {
                        self.index += 1;
                        return Ok(());
                    }
                }
                Some(b'}') => {
                    self.index += 1;
                    return Ok(());
                }
                None => return Err(Json5ScanError::UnterminatedObject),
                _ => return Err(Json5ScanError::ExpectedCommaOrEnd(self.index)),
            }
        }
    }

    fn parse_value(&mut self, parent_depth: usize) -> Result<(), Json5ScanError> {
        self.skip_trivia()?;
        match self.bytes.get(self.index).copied() {
            Some(b'{') => {
                let mut ignored_keys = Vec::new();
                self.parse_object(parent_depth + 1, false, &mut ignored_keys)
            }
            Some(b'[') => self.parse_array(parent_depth + 1),
            Some(b'\'' | b'"') => self.parse_string().map(|_| ()),
            Some(_) => self.parse_number_or_literal(),
            None => Err(Json5ScanError::UnexpectedValue(self.index)),
        }
    }
}

fn valid_json5_number(token: &str) -> bool {
    let unsigned = token
        .strip_prefix('+')
        .or_else(|| token.strip_prefix('-'))
        .unwrap_or(token);
    if unsigned.is_empty()
        || unsigned
            .as_bytes()
            .first()
            .is_some_and(|first| matches!(*first, b'+' | b'-'))
    {
        return false;
    }

    if let Some(hex) = unsigned
        .strip_prefix("0x")
        .or_else(|| unsigned.strip_prefix("0X"))
    {
        return !hex.is_empty() && hex.bytes().all(|byte| byte.is_ascii_hexdigit());
    }

    let mut exponent_parts = unsigned.split(|character| matches!(character, 'e' | 'E'));
    let Some(mantissa) = exponent_parts.next() else {
        return false;
    };
    let exponent = exponent_parts.next();
    if exponent_parts.next().is_some() {
        return false;
    }

    if let Some(exponent) = exponent {
        let digits = exponent
            .strip_prefix('+')
            .or_else(|| exponent.strip_prefix('-'))
            .unwrap_or(exponent);
        if digits.is_empty()
            || digits
                .as_bytes()
                .first()
                .is_some_and(|first| matches!(*first, b'+' | b'-'))
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return false;
        }
    }

    let mut decimal_parts = mantissa.split('.');
    let Some(integer) = decimal_parts.next() else {
        return false;
    };
    let fraction = decimal_parts.next();
    if decimal_parts.next().is_some() {
        return false;
    }

    let integer_valid = integer.bytes().all(|byte| byte.is_ascii_digit());
    if !integer_valid {
        return false;
    }

    match fraction {
        None => !integer.is_empty(),
        Some(fraction) => {
            fraction.bytes().all(|byte| byte.is_ascii_digit())
                && (!integer.is_empty() || !fraction.is_empty())
        }
    }
}

/// Returns top-level object keys from JSON, JSONC, or JSON5 while validating
/// the complete document and enforcing a finite nesting depth.
pub fn top_level_keys(input: &str) -> Result<Vec<String>, Json5ScanError> {
    let mut scanner = Scanner::new(input);
    scanner.skip_trivia()?;
    if scanner.bytes.get(scanner.index) != Some(&b'{') {
        return Err(Json5ScanError::ExpectedObject(scanner.index));
    }

    let mut keys = Vec::new();
    scanner.parse_object(0, true, &mut keys)?;
    scanner.skip_trivia()?;
    if scanner.index != scanner.bytes.len() {
        return Err(Json5ScanError::TrailingContent(scanner.index));
    }
    Ok(keys)
}
