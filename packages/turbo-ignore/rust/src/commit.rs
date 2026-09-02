use crate::{CommitDecision, CommitResult, CommitScope, validate_workspace};

pub const SKIP_ALL_COMMITS: [&str; 5] = [
    "[skip ci]",
    "[ci skip]",
    "[no ci]",
    "[skip vercel]",
    "[vercel skip]",
];

pub const FORCE_ALL_COMMITS: [&str; 2] = ["[vercel deploy]", "[vercel build]"];
pub const ONLY_WORKSPACE_PREFIX: &str = "[vercel only ";

fn bracketed_directives(commit_message: &str) -> Vec<&str> {
    let bytes = commit_message.as_bytes();
    let mut directives = Vec::new();
    let mut index = 0_usize;

    while index < bytes.len() {
        let Some(relative_start) = bytes[index..].iter().position(|byte| *byte == b'[') else {
            break;
        };
        let start = index + relative_start;
        let Some(relative_end) = bytes[start + 1..].iter().position(|byte| *byte == b']') else {
            break;
        };
        let end = start + 1 + relative_end;
        if let Some(value) = commit_message.get(start..=end) {
            directives.push(value);
        }
        index = end + 1;
    }

    directives
}

fn found(result: CommitResult, scope: CommitScope, directive: &str) -> CommitDecision {
    CommitDecision {
        result,
        scope,
        reason: format!("Found commit message: {directive}"),
    }
}

fn conflict(scope: CommitScope, first: &str, second: &str) -> CommitDecision {
    CommitDecision {
        result: CommitResult::Conflict,
        scope,
        reason: format!("Conflicting commit messages found: {first} and {second}"),
    }
}

/// Parses Vercel deployment directives.
///
/// For a single directive this matches the TypeScript implementation. Multiple
/// `[vercel only ...]` directives are treated as a conflict instead of using
/// the TypeScript implementation's greedy regular expression. The engine
/// resolves every conflict to deployment, so ambiguity cannot suppress a
/// required build.
#[must_use]
pub fn check_commit(workspace: &str, commit_message: &str) -> CommitDecision {
    let directives = bracketed_directives(commit_message);
    let workspace_is_safe = validate_workspace(workspace).is_ok();
    let workspace_directive = directives.iter().copied().find(|directive| {
        directive.starts_with(ONLY_WORKSPACE_PREFIX)
            || directive.starts_with("[vercel deploy ")
            || directive.starts_with("[vercel build ")
            || directive.starts_with("[vercel skip ")
    });
    if !workspace_is_safe && workspace_directive.is_some() {
        return CommitDecision {
            result: CommitResult::Conflict,
            scope: CommitScope::Workspace,
            reason: "Unsafe workspace prevents reliable commit-directive matching".to_owned(),
        };
    }

    let only_directives: Vec<&str> = directives
        .iter()
        .copied()
        .filter(|directive| directive.starts_with(ONLY_WORKSPACE_PREFIX))
        .collect();

    if only_directives.len() > 1 {
        return conflict(
            CommitScope::Workspace,
            only_directives[0],
            only_directives[1],
        );
    }

    if let Some(directive) = only_directives.first().copied() {
        let expected = format!("[vercel only {workspace}]");
        if directive == expected {
            return found(CommitResult::Deploy, CommitScope::Workspace, directive);
        }
        return found(CommitResult::Skip, CommitScope::Workspace, directive);
    }

    let workspace_deploy = [
        format!("[vercel deploy {workspace}]"),
        format!("[vercel build {workspace}]"),
    ];
    let workspace_skip = format!("[vercel skip {workspace}]");
    let found_workspace_deploy = workspace_deploy
        .iter()
        .find(|directive| commit_message.contains(directive.as_str()));
    let found_workspace_skip = commit_message
        .contains(&workspace_skip)
        .then_some(workspace_skip.as_str());

    if let (Some(deploy), Some(skip)) = (found_workspace_deploy, found_workspace_skip) {
        return conflict(CommitScope::Workspace, deploy, skip);
    }
    if let Some(deploy) = found_workspace_deploy {
        return found(CommitResult::Deploy, CommitScope::Workspace, deploy);
    }
    if let Some(skip) = found_workspace_skip {
        return found(CommitResult::Skip, CommitScope::Workspace, skip);
    }

    let force_deploy = FORCE_ALL_COMMITS
        .iter()
        .copied()
        .find(|directive| commit_message.contains(*directive));
    let force_skip = SKIP_ALL_COMMITS
        .iter()
        .copied()
        .find(|directive| commit_message.contains(*directive));

    if let (Some(deploy), Some(skip)) = (force_deploy, force_skip) {
        return conflict(CommitScope::Global, deploy, skip);
    }
    if let Some(deploy) = force_deploy {
        return found(CommitResult::Deploy, CommitScope::Global, deploy);
    }
    if let Some(skip) = force_skip {
        return found(CommitResult::Skip, CommitScope::Global, skip);
    }

    CommitDecision {
        result: CommitResult::Continue,
        scope: CommitScope::Global,
        reason: "No deploy or skip string found in commit message.".to_owned(),
    }
}
