// PACKAGE_MANAGER_DECLARATION_RED_START

pub const PACKAGE_MANAGER_FIELD_INPUT_LIMIT: usize = 512;
pub const PACKAGE_MANAGER_VERSION_INPUT_LIMIT: usize = 256;
pub const PACKAGE_MANAGER_RANGE_DISJUNCT_LIMIT: usize = 32;

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

pub fn get_workspace_package_manager(
    _manifest: &PackageManagerManifestInput,
) -> Result<Option<WorkspaceManager>, PackageManagerDeclarationError> {
    Ok(None)
}

pub fn set_package_manager_declaration(
    _document: &mut PackageManagerDeclarationDocument,
    _manager: WorkspaceManager,
    _version: &str,
) -> Result<(), PackageManagerDeclarationError> {
    Err(PackageManagerDeclarationError {
        message: "package-manager declaration behavioral RED".to_owned(),
    })
}
