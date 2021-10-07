use serde::{Deserialize, Serialize};

/// Dove manifest.
#[derive(Deserialize, Serialize, Debug, Clone, Default, Eq, PartialEq)]
pub struct DoveToml {
    /// Project info.
    pub package: Package,
}

/// Project info.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Package {
    /// Project name.
    pub name: Option<String>,
    /// Project account address.
    pub account_address: Option<String>,
    /// Minimal dove version.
    pub dove_version: Option<String>,
    /// Dialect
    pub dialect: Option<String>,
}

impl Default for Package {
    fn default() -> Self {
        Package { name: None, account_address: None, dove_version: None, dialect: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_dove_toml_package() {
        let text = r#"
[package]
name = "move_project"
account_address = "0x1"
dove_version = "0.11.0"
dialect = "pont"
        "#;

        let dove_toml: DoveToml = toml::from_str(text).unwrap();
        assert_eq!(dove_toml.package.name, Some("move_project".to_string()));
        assert_eq!(dove_toml.package.account_address, Some("0x1".to_string()));
        assert_eq!(dove_toml.package.dove_version, Some("0.11.0".to_string()));
        assert_eq!(dove_toml.package.dialect, Some("pont".to_string()));
    }
}
