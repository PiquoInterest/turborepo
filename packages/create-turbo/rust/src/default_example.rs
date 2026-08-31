pub const DEFAULT_EXAMPLES: [&str; 2] = ["basic", "default"];

pub fn is_default_example(example: &str) -> bool {
    matches!(example, "basic" | "default")
}
