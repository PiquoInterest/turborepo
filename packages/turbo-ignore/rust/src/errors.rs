use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorLevel {
    Warn,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    MissingLockfile,
    NoPackageManager,
    UnreachableParent,
    InvalidComparison,
    UnknownError,
}

impl ErrorCode {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingLockfile => "MISSING_LOCKFILE",
            Self::NoPackageManager => "NO_PACKAGE_MANAGER",
            Self::UnreachableParent => "UNREACHABLE_PARENT",
            Self::InvalidComparison => "INVALID_COMPARISON",
            Self::UnknownError => "UNKNOWN_ERROR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorClassification {
    pub level: ErrorLevel,
    pub message: String,
    pub code: ErrorCode,
}

fn matches(pattern: &str, value: &str) -> bool {
    Regex::new(pattern)
        .map(|regex| regex.is_match(value))
        .unwrap_or(false)
}

#[must_use]
pub fn classify_error(error: &str) -> ErrorClassification {
    if matches(
        r"(?i)reading (?:yarn\.lock|package-lock\.json|pnpm-lock\.yaml):.*?no such file or directory",
        error,
    ) {
        return ErrorClassification {
            level: ErrorLevel::Warn,
            code: ErrorCode::MissingLockfile,
            message: "turbo-ignore could not complete - no lockfile found, please commit one to \
                      your repository"
                .to_owned(),
        };
    }

    if matches(
        r"(?i)run failed: We did not detect an in-use package manager for your project",
        error,
    ) || matches(
        r#"(?i)run failed: We did not find a package manager specified in your root package\.json"#,
        error,
    ) {
        return ErrorClassification {
            level: ErrorLevel::Warn,
            code: ErrorCode::NoPackageManager,
            message: "turbo-ignore could not complete - no package manager detected, please \
                      commit a lockfile, or set \"devEngines.packageManager\" in your root \
                      \"package.json\""
                .to_owned(),
        };
    }

    if matches(
        r"(?i)failed to resolve packages to run: commit HEAD\^ does not exist",
        error,
    ) {
        return ErrorClassification {
            level: ErrorLevel::Warn,
            code: ErrorCode::UnreachableParent,
            message: "turbo-ignore could not complete - parent commit does not exist or is \
                      unreachable"
                .to_owned(),
        };
    }

    if matches(r"(?i)commit \S+ does not exist", error)
        || matches(r"(?i)unknown revision", error)
        || matches(r"(?i)invalid symmetric difference expression", error)
    {
        return ErrorClassification {
            level: ErrorLevel::Warn,
            code: ErrorCode::InvalidComparison,
            message: "turbo-ignore could not complete - a ref or SHA is invalid. It could have \
                      been removed from the branch history via a force push, or this could be a \
                      shallow clone with insufficient history"
                .to_owned(),
        };
    }

    ErrorClassification {
        level: ErrorLevel::Error,
        code: ErrorCode::UnknownError,
        message: error.to_owned(),
    }
}
