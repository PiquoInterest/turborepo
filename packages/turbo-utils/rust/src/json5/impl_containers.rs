impl<'a> Parser<'a> {
    fn parse_array(&mut self, depth: usize) -> Result<Value, Json5Error> {
        if depth > MAX_NESTING_DEPTH {
            return Err(Json5Error::NestingTooDeep);
        }
        self.index += 1;
        self.skip_trivia()?;
        let mut values = Vec::new();
        if self.bytes.get(self.index) == Some(&b']') {
            self.index += 1;
            return Ok(Value::Array(values));
        }

        loop {
            values.push(self.parse_value(depth)?);
            self.skip_trivia()?;
            match self.bytes.get(self.index).copied() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_trivia()?;
                    if self.bytes.get(self.index) == Some(&b']') {
                        self.index += 1;
                        return Ok(Value::Array(values));
                    }
                }
                Some(b']') => {
                    self.index += 1;
                    return Ok(Value::Array(values));
                }
                None => return Err(Json5Error::UnexpectedEnd(self.index)),
                _ => return Err(Json5Error::ExpectedCommaOrEnd(self.index)),
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Value, Json5Error> {
        if depth > MAX_NESTING_DEPTH {
            return Err(Json5Error::NestingTooDeep);
        }
        self.index += 1;
        self.skip_trivia()?;
        let mut values = Map::new();
        if self.bytes.get(self.index) == Some(&b'}') {
            self.index += 1;
            return Ok(Value::Object(values));
        }

        loop {
            let key = self.parse_key()?;
            self.skip_trivia()?;
            if self.bytes.get(self.index) != Some(&b':') {
                return Err(Json5Error::ExpectedColon(self.index));
            }
            self.index += 1;
            self.skip_trivia()?;
            let value = self.parse_value(depth)?;
            values.insert(key, value);
            self.skip_trivia()?;

            match self.bytes.get(self.index).copied() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_trivia()?;
                    if self.bytes.get(self.index) == Some(&b'}') {
                        self.index += 1;
                        return Ok(Value::Object(values));
                    }
                }
                Some(b'}') => {
                    self.index += 1;
                    return Ok(Value::Object(values));
                }
                None => return Err(Json5Error::UnexpectedEnd(self.index)),
                _ => return Err(Json5Error::ExpectedCommaOrEnd(self.index)),
            }
        }
    }

    fn parse_value(&mut self, parent_depth: usize) -> Result<Value, Json5Error> {
        self.skip_trivia()?;
        match self.bytes.get(self.index).copied() {
            Some(b'{') => self.parse_object(parent_depth + 1),
            Some(b'[') => self.parse_array(parent_depth + 1),
            Some(b'\'' | b'"') => self.parse_string().map(Value::String),
            Some(_) => self.parse_atom(),
            None => Err(Json5Error::UnexpectedEnd(self.index)),
        }
    }
}
