impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            input,
            bytes: input.as_bytes(),
            index: 0,
        }
    }

    fn skip_trivia(&mut self) -> Result<(), Json5Error> {
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
                    return Err(Json5Error::UnterminatedComment(start));
                }
                continue;
            }

            return Ok(());
        }
    }

}
