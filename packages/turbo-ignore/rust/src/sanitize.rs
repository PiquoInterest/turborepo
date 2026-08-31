const MAX_LOG_VALUE_CHARS: usize = 2_048;

/// Escapes control characters before user-controlled values reach terminal logs.
#[must_use]
pub fn sanitize_for_log(value: &str) -> String {
    let mut output = String::with_capacity(value.len().min(MAX_LOG_VALUE_CHARS));
    let mut truncated = false;

    for (index, character) in value.chars().enumerate() {
        if index >= MAX_LOG_VALUE_CHARS {
            truncated = true;
            break;
        }

        match character {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{1b}' => output.push_str("\\u{1b}"),
            value if value.is_control() => {
                use std::fmt::Write as _;
                let _write_result = write!(output, "\\u{{{:x}}}", value as u32);
            }
            value => output.push(value),
        }
    }

    if truncated {
        output.push('…');
    }

    output
}
