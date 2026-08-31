impl<'a> Parser<'a> {
    fn parse_atom(&mut self) -> Result<Value, Json5Error> {
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
                return Err(Json5Error::UnexpectedValue(self.index));
            }
            self.index += 1;
        }

        let token = self
            .input
            .get(start..self.index)
            .ok_or(Json5Error::UnexpectedValue(start))?;
        match token {
            "true" => Ok(Value::Bool(true)),
            "false" => Ok(Value::Bool(false)),
            "null" => Ok(Value::Null),
            "Infinity" | "+Infinity" | "-Infinity" | "NaN" | "+NaN" | "-NaN" => {
                Err(Json5Error::NonFiniteNumber(start))
            }
            _ => parse_number(token, start).map(Value::Number),
        }
    }

}
