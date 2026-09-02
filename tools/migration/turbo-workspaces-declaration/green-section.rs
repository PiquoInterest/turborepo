// PACKAGE_MANAGER_DECLARATION_GREEN_START

pub const PACKAGE_MANAGER_FIELD_INPUT_LIMIT: usize = 512;
pub const PACKAGE_MANAGER_VERSION_INPUT_LIMIT: usize = 256;
pub const PACKAGE_MANAGER_RANGE_DISJUNCT_LIMIT: usize = 32;
const JAVASCRIPT_MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum JsonField<T> {
    #[default]
    Missing,
    Null,
    Array,
    Other,
    Value(T),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum JsonString {
    #[default]
    Missing,
    Value(String),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevEnginesPackageManagerInput {
    pub name: JsonString,
    pub version: JsonString,
    pub property_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevEnginesInput {
    pub package_manager: JsonField<DevEnginesPackageManagerInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageManagerManifestInput {
    pub package_manager: JsonString,
    pub dev_engines: JsonField<DevEnginesInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerDeclarationError {
    message: String,
}

impl PackageManagerDeclarationError {
    #[must_use]
    pub const fn error_type(&self) -> &'static str {
        "package_manager-unable_to_detect"
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for PackageManagerDeclarationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerDeclaration {
    pub name: WorkspaceManager,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevEnginesDeclarationDocument {
    pub package_manager: Option<PackageManagerDeclaration>,
    pub preserved_property_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PackageManagerDeclarationDocument {
    pub package_manager: Option<String>,
    pub dev_engines: JsonField<DevEnginesDeclarationDocument>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedVersion {
    major: u64,
    minor: u64,
    patch: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RangeError {
    Invalid,
    Empty,
    MultipleMajors,
    TooComplex,
    Unsafe,
}

pub fn get_workspace_package_manager(
    manifest: &PackageManagerManifestInput,
) -> Result<Option<WorkspaceManager>, PackageManagerDeclarationError> {
    if let JsonString::Value(package_manager) = &manifest.package_manager {
        if !package_manager.is_empty() {
            validate_bounded_policy_text(
                "packageManager",
                package_manager,
                PACKAGE_MANAGER_FIELD_INPUT_LIMIT,
            )?;
            return Ok(parse_legacy_package_manager(package_manager));
        }
    }

    let dev_engines = match &manifest.dev_engines {
        JsonField::Missing => return Ok(None),
        JsonField::Value(value) => value,
        JsonField::Null | JsonField::Array | JsonField::Other => {
            return Err(invalid_dev_engines(
                "`devEngines` must be an object containing `packageManager`",
            ));
        }
    };

    let declaration = match &dev_engines.package_manager {
        JsonField::Missing => return Ok(None),
        JsonField::Value(value) => value,
        JsonField::Null | JsonField::Array | JsonField::Other => {
            return Err(invalid_dev_engines(
                "`devEngines.packageManager` must be an object",
            ));
        }
    };

    if declaration.property_count == 0 {
        return Err(invalid_dev_engines(
            "expected `{ \"name\": \"pnpm\", \"version\": \"9.12.3\" }`",
        ));
    }

    let name = match &declaration.name {
        JsonString::Missing => {
            return Err(invalid_dev_engines(
                "`devEngines.packageManager.name` is required",
            ));
        }
        JsonString::Other => {
            return Err(invalid_dev_engines(
                "`devEngines.packageManager.name` must be a string",
            ));
        }
        JsonString::Value(value) => value,
    };

    if name.is_empty() {
        return Err(invalid_dev_engines(
            "`devEngines.packageManager.name` must not be empty",
        ));
    }
    if name.trim() != name {
        return Err(invalid_dev_engines(
            "`devEngines.packageManager.name` must not contain leading or trailing whitespace",
        ));
    }
    validate_bounded_policy_text(
        "devEngines.packageManager.name",
        name,
        PACKAGE_MANAGER_FIELD_INPUT_LIMIT,
    )?;
    let Some(manager) = parse_manager(name) else {
        return Err(invalid_dev_engines(
            "`devEngines.packageManager.name` must be one of `npm`, `pnpm`, `yarn`, `bun`, `nub`, or `aube`",
        ));
    };

    let version = match &declaration.version {
        JsonString::Missing => {
            return Err(invalid_dev_engines(
                "`devEngines.packageManager.version` is required",
            ));
        }
        JsonString::Other => {
            return Err(invalid_dev_engines(
                "`devEngines.packageManager.version` must be a string",
            ));
        }
        JsonString::Value(value) => value,
    };

    if version.is_empty() {
        return Err(invalid_dev_engines(
            "`devEngines.packageManager.version` must not be empty",
        ));
    }
    if version.trim() != version {
        return Err(invalid_dev_engines(
            "`devEngines.packageManager.version` must not contain leading or trailing whitespace",
        ));
    }
    validate_bounded_policy_text(
        "devEngines.packageManager.version",
        version,
        PACKAGE_MANAGER_VERSION_INPUT_LIMIT,
    )?;

    match analyze_version_range(version) {
        Ok(_) => Ok(Some(manager)),
        Err(RangeError::Invalid) => Err(invalid_dev_engines(
            "`devEngines.packageManager.version` must be a valid semantic version range",
        )),
        Err(RangeError::Empty) => Err(invalid_dev_engines(
            "`devEngines.packageManager.version` must admit at least one version",
        )),
        Err(RangeError::MultipleMajors) => Err(invalid_dev_engines(
            "`devEngines.packageManager.version` must only allow versions within one major version",
        )),
        Err(RangeError::TooComplex) => Err(invalid_dev_engines(
            "`devEngines.packageManager.version` contains too many range disjuncts",
        )),
        Err(RangeError::Unsafe) => Err(invalid_dev_engines(
            "`devEngines.packageManager.version` contains unsafe or unsupported text",
        )),
    }
}

pub fn set_package_manager_declaration(
    document: &mut PackageManagerDeclarationDocument,
    manager: WorkspaaceManager,
    version: &str,
) -> Result<(), PackageManagerDeclarationError> {
    validate_bounded_policy_text(
        "package manager version",
        version,
        PACKAGE_MANAGER_VERSION_INPUT_LIMIT,
    )?;
    let normalized_version = normalize_exact_version(version).ok_or_else(|| {
        PackageManagerDeclarationError {
            message: "Invalid package manager version: expected an exact semantic version".to_owned(),
        }
    })?;

    let mut dev_engines = match &document.dev_engines {
        JsonField::Value(existing) => existing.clone(),
        JsonField::Missing | JsonField::Null | JsonField::Array | JsonField::Other => {
            DevEnginesDeclarationDocument::default()
        }
    };
    dev_engines.package_manager = Some(PackageManagerDeclaration {
        name: manager,
        version: normalized_version,
    });

    document.package_manager = None;
    document.dev_engines = JsonField::Value(dev_engines);
    Ok(())
}
