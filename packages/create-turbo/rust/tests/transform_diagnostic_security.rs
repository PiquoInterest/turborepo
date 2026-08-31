use create_turbo_rs::{
    MAX_TERMINAL_DIAGNOSTIC_SCALARS, TransformFailure, sanitize_terminal_text,
};

#[test]
fn terminal_rendering_neutralizes_controls_and_bidirectional_formatting() {
    let raw_message =
        "failed\u{001b}[31m\nnext\rline\tcol\u{202e}txt\u{2066}iso\u{200b}hidden\u{009b}csi";
    let raw_transform = "../../official-starter\0\u{2069}";
    let expected_message =
        "failed\\u{1b}[31m\\nnext\\rline\\tcol\\u{202e}txt\\u{2066}iso\\u{200b}hidden\\u{9b}csi";
    let expected_transform = "../../official-starter\\0\\u{2069}";

    let failure =
        TransformFailure::with_options(raw_message, Some(raw_transform), Some(false));

    assert_eq!(sanitize_terminal_text(raw_message), expected_message);
    assert_eq!(failure.to_string(), expected_message);
    assert_eq!(failure.terminal_transform(), expected_transform);
    assert_eq!(failure.message, raw_message);
    assert_eq!(failure.transform, raw_transform);
    assert!(!failure.fatal);
}

#[test]
fn terminal_rendering_is_bounded_for_attacker_controlled_text() {
    let raw = "x".repeat(MAX_TERMINAL_DIAGNOSTIC_SCALARS + 4096);
    let rendered = sanitize_terminal_text(&raw);

    assert_eq!(
        rendered.chars().count(),
        MAX_TERMINAL_DIAGNOSTIC_SCALARS + 1
    );
    assert!(rendered.ends_with('…'));
    assert!(rendered.len() < raw.len());
}

#[test]
fn ordinary_printable_unicode_is_preserved_exactly() {
    let text = "Unable to transform café 🚀";
    assert_eq!(sanitize_terminal_text(text), text);
}
