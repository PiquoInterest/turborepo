/// Parses the JSON, JSONC, and JSON5 forms accepted by the TypeScript utility.
///
/// Configuration input is bounded and recursion depth is finite. Non-finite
/// JSON5 numbers are rejected because they cannot represent a valid Turbo
/// schema value and `serde_json::Value` intentionally has no such state.
pub fn parse_json5(input: &str) -> Result<Value, Json5Error> {
    if input.len() > MAX_JSON5_BYTES {
        return Err(Json5Error::InputTooLarge);
    }

    let mut parser = Parser::new(input);
    parser.skip_trivia()?;
    let value = parser.parse_value(0)?;
    parser.skip_trivia()?;
    if parser.index != parser.bytes.len() {
        return Err(Json5Error::TrailingContent(parser.index));
    }
    Ok(value)
}
