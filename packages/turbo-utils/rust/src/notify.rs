use thiserror::Error;

pub const NOTIFY_MAX_UNTRUSTED_CHARS: usize = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateInfo {
    pub latest: String,
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct UpdateCheckError {
    message: String,
}

impl UpdateCheckError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait UpdateChecker: Sync {
    fn check(&self, package_info: &PackageInfo) -> Result<Option<UpdateInfo>, UpdateCheckError>;
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct UpgradeCommandError {
    message: String,
}

impl UpgradeCommandError {
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

pub trait UpgradeCommandProvider: Sync {
    fn resolve(&self) -> Result<Option<String>, UpgradeCommandError>;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(i32)]
pub enum ExitCode {
    #[default]
    Success = 0,
    Failure = 1,
}

impl ExitCode {
    #[must_use]
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

pub enum UpgradeCommand<'a> {
    None,
    Static(&'a str),
    Dynamic(&'a dyn UpgradeCommandProvider),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyUpdateOutcome {
    pub exit_code: ExitCode,
    pub stdout: Vec<String>,
    pub stderr: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUpdateNotification {
    package_info: PackageInfo,
    update: Option<UpdateInfo>,
}

impl PreparedUpdateNotification {
    #[must_use]
    pub fn prepare<C: UpdateChecker + ?Sized>(package_info: PackageInfo, checker: &C) -> Self {
        let update = checker.check(&package_info).ok().flatten();
        Self {
            package_info,
            update,
        }
    }

    #[must_use]
    pub fn notify(
        &self,
        exit_code: ExitCode,
        upgrade_command: UpgradeCommand<'_>,
        debug: bool,
    ) -> NotifyUpdateOutcome {
        let mut outcome = NotifyUpdateOutcome {
            exit_code,
            stdout: Vec::new(),
            stderr: Vec::new(),
        };
        let Some(update) = &self.update else {
            return outcome;
        };
        if update.latest.is_empty() {
            return outcome;
        }

        outcome.stdout.push(String::new());
        outcome.stdout.push(format!(
            "A new version of `{}` is available!",
            escape_untrusted(&self.package_info.name)
        ));

        let resolved = match upgrade_command {
            UpgradeCommand::None => Ok(None),
            UpgradeCommand::Static(command) => Ok(Some(command.to_owned())),
            UpgradeCommand::Dynamic(provider) => provider.resolve(),
        };
        match resolved {
            Ok(Some(command)) if !command.is_empty() => outcome.stdout.push(format!(
                "You can update by running: {}",
                escape_untrusted(&command)
            )),
            Ok(_) => {}
            Err(error) => {
                if debug {
                    outcome.stderr.push(format!(
                        "Update check failed: {}",
                        escape_untrusted(&error.to_string())
                    ));
                }
                return outcome;
            }
        }

        outcome.stdout.push(String::new());
        outcome
    }
}

fn is_directional_format_control(character: char) -> bool {
    matches!(
        character,
        '\u{061c}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
    )
}

fn escape_untrusted(value: &str) -> String {
    let mut escaped = String::new();
    let mut truncated = false;
    for (consumed, character) in value.chars().enumerate() {
        if consumed == NOTIFY_MAX_UNTRUSTED_CHARS {
            truncated = true;
            break;
        }
        match character {
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{001b}' => escaped.push_str("\\x1b"),
            character if character.is_control() || is_directional_format_control(character) => {
                escaped.push_str(&format!("\\u{{{:x}}}", u32::from(character)));
            }
            character => escaped.push(character),
        }
    }
    if truncated {
        escaped.push('…');
    }
    escaped
}
