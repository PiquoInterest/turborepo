use create_turbo_rs::is_default_example;

#[test]
fn prefixes_and_suffixes_cannot_select_the_default_acquisition_path() {
    for example in ["basic-extra", "prebasic", "default/example", "xdefault"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn control_characters_and_nul_are_rejected_by_exact_membership() {
    for example in ["basic\0", "default\0suffix", "basic\r", "default\u{7f}"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn unicode_confusables_do_not_match_ascii_default_names() {
    for example in ["basıc", "baѕic", "defauⅼt", "ｄｅｆａｕｌｔ"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn unicode_normalization_is_not_applied_implicitly() {
    for example in ["básic", "défault", "de\u{200d}fault"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn large_untrusted_names_are_rejected_without_copying_or_scanning_a_collection() {
    let example = "x".repeat(4 * 1024 * 1024);
    assert!(!is_default_example(&example));
}
