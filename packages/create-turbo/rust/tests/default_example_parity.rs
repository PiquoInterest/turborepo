use create_turbo_rs::{DEFAULT_EXAMPLES, is_default_example};

#[test]
fn exported_default_examples_preserve_source_iteration_order() {
    assert_eq!(DEFAULT_EXAMPLES, ["basic", "default"]);
}

#[test]
fn basic_is_a_default_example() {
    assert!(is_default_example("basic"));
}

#[test]
fn default_is_a_default_example() {
    assert!(is_default_example("default"));
}

#[test]
fn matching_is_case_sensitive_like_javascript_set_membership() {
    for example in ["Basic", "BASIC", "Default", "DEFAULT"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn matching_does_not_trim_whitespace() {
    for example in [" basic", "basic ", "\tdefault", "default\n"] {
        assert!(!is_default_example(example));
    }
}

#[test]
fn empty_and_non_default_examples_are_rejected() {
    for example in ["", "with-tailwind", "kitchen-sink", "examples/basic"] {
        assert!(!is_default_example(example));
    }
}
