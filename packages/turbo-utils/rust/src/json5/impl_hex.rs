impl<'a> Parser<'a> {
    fn parse_hex_u16(&mut self, start: usize) -> Result<u16, Json5Error> {
        let mut value = 0_u16;
        for _ in 0..4 {
            let digit = *self
                .bytes
                .get(self.index)
                .ok_or(Json5Error::InvalidUnicodeEscape(start))?;
            let digit = match digit {
                b'0'..=b'9' => u16::from(digit - b'0'),
                b'a'..=b'f' => u16::from(digit - b'a') + 10,
                b'A'..=b'F' => u16::from(digit - b'A') + 10,
                _ => return Err(Json5Error::InvalidUnicodeEscape(start)),
            };
            value = value * 16 + digit;
            self.index += 1;
        }
        Ok(value)
    }

    fn parse_hex_byte(&mut self, start: usize) -> Result<u8, Json5Error> {
        let mut value = 0_u8;
        for _ in 0..2 {
            let digit = *self
                .bytes
                .get(self.index)
                .ok_or(Json5Error::InvalidEscape(start))?;
            let digit = match digit {
                b'0'..=b'9' => digit - b'0',
                b'a'..=b'f' => digit - b'a' + 10,
                b'A'..=b'F' => digit - b'A' + 10,
                _ => return Err(Json5Error::InvalidEscape(start)),
            };
            value = value * 16 + digit;
            self.index += 1;
        }
        Ok(value)
    }

    fn parse_unicode_escape(&mut self, start: usize) -> Result<char, Json5Error> {
        let first = self.parse_hex_u16(start)?;
        let code_point = if (0xD800..=0xDBFF).contains(&first) {
            if self.bytes.get(self.index) != Some(&b'\\')
                || self.bytes.get(self.index + 1) != Some(&b'u')
            {
                return Err(Json5Error::InvalidUnicodeEscape(start));
            }
            self.index += 2;
            let second = self.parse_hex_u16(start)?;
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Err(Json5Error::InvalidUnicodeEscape(start));
            }
            0x1_0000 + (u32::from(first - 0xD800) << 10) + u32::from(second - 0xDC00)
        } else if (0xDC00..=0xDFFF).contains(&first) {
            return Err(Json5Error::InvalidUnicodeEscape(start));
        } else {
            u32::from(first)
        };

        char::from_u32(code_point).ok_or(Json5Error::InvalidUnicodeEscape(start))
    }

}
