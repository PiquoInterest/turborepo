impl<'a> Parser<'a> {
    fn parse_string(&mut self) -> Result<String, Json5Error> {
        let start = self.index;
        let quote = *self
            .bytes
            .get(self.index)
            .ok_or(Json5Error::UnterminatedString(start))?;
        self.index += 1;
        let mut output = String::new();

        while let Some(value) = self.bytes.get(self.index).copied() {
            if value == quote {
                self.index += 1;
                return Ok(output);
            }
            if value < 0x20 {
                return Err(Json5Error::UnterminatedString(start));
            }
            if value != b'\\' {
                let remainder = self
                    .input
                    .get(self.index..)
                    .ok_or(Json5Error::UnterminatedString(start))?;
                let character = remainder
                    .chars()
                    .next()
                    .ok_or(Json5Error::UnterminatedString(start))?;
                output.push(character);
                self.index += character.len_utf8();
                continue;
            }

            let escape_start = self.index;
            self.index += 1;
            let escaped = *self
                .bytes
                .get(self.index)
                .ok_or(Json5Error::UnterminatedString(start))?;
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
                        return Err(Json5Error::InvalidEscape(escape_start));
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
                _ => return Err(Json5Error::InvalidEscape(escape_start)),
            }
        }

        Err(Json5Error::UnterminatedString(start))
    }

    fn parse_identifier(&mut self) -> Result<String, Json5Error> {
        let start = self.index;
        let first = self
            .bytes
            .get(self.index)
            .copied()
            .ok_or(Json5Error::ExpectedKey(start))?;
        if !(first.is_ascii_alphabetic() || matches!(first, b'_' | b'$')) {
            return Err(Json5Error::ExpectedKey(start));
        }
        self.index += 1;
        while self.bytes.get(self.index).is_some_and(|value| {
            value.is_ascii_alphanumeric() || matches!(*value, b'_' | b'$')
        }) {
            self.index += 1;
        }
        Ok(self.input[start..self.index].to_owned())
    }

    fn parse_key(&mut self) -> Result<String, Json5Error> {
        match self.bytes.get(self.index).copied() {
            Some(b'\'' | b'"') => self.parse_string(),
            Some(value) if value.is_ascii_alphabetic() || matches!(value, b'_' | b'$') => {
                self.parse_identifier()
            }
            _ => Err(Json5Error::ExpectedKey(self.index)),
        }
    }

}
