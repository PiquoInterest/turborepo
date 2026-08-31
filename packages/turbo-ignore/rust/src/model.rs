use std::fmt;

pub const EXIT_SKIP: i32 = 0;
pub const EXIT_DEPLOY: i32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildDecision {
    Skip,
    Deploy,
}

impl BuildDecision {
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Skip => EXIT_SKIP,
            Self::Deploy => EXIT_DEPLOY,
        }
    }
}

impl fmt::Display for BuildDecision {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Skip => f.write_str("skip"),
            Self::Deploy => f.write_str("deploy"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitResult {
    Skip,
    Deploy,
    Continue,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitScope {
    Global,
    Workspace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitDecision {
    pub result: CommitResult,
    pub scope: CommitScope,
    pub reason: String,
}
