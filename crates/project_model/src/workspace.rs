use anyhow::{format_err, Context, Result};
use std::fs;

use crate::dove_toml::DoveToml;
use crate::ProjectManifest;
use paths::AbsPathBuf;

/// `PackageRoot` describes a package root folder.
/// Which may be an external dependency, or a member of
/// the current workspace.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct PackageRoot {
    /// Is from the local filesystem and may be edited
    pub is_local: bool,
    pub include: Vec<AbsPathBuf>,
    pub exclude: Vec<AbsPathBuf>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum ProjectWorkspace {
    Dove { dove_toml: DoveToml },
    DetachedFiles { files: Vec<AbsPathBuf> },
}

impl ProjectWorkspace {
    pub fn load(manifest: ProjectManifest) -> Result<ProjectWorkspace> {
        let ws = match manifest {
            ProjectManifest::DoveToml(dove_toml_path) => {
                let file_text = fs::read_to_string(&dove_toml_path).with_context(|| {
                    format!("Failed to read Dove.toml file {}", dove_toml_path.display())
                })?;
                let dove_toml = toml::from_str(&file_text).with_context(|| {
                    format!("Failed to deserialize Dove.toml file {}", dove_toml_path.display())
                })?;
                ProjectWorkspace::Dove { dove_toml }
            }
        };
        Ok(ws)
    }

    pub fn load_detached_files(detached_files: Vec<AbsPathBuf>) -> Result<ProjectWorkspace> {
        Ok(ProjectWorkspace::DetachedFiles { files: detached_files })
    }
}
