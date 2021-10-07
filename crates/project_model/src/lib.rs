#![allow(unused)]

use anyhow::{bail, format_err, Context, Result};

use crate::manifest_path::ManifestPath;
use paths::{AbsPath, AbsPathBuf};
use rustc_hash::FxHashSet;
use std::convert::{TryFrom, TryInto};
use std::fs::{read_dir, ReadDir};
use std::process::Command;
use std::{fs, io};

mod manifest_path;
mod dove_toml;
mod workspace;

pub use workspace::ProjectWorkspace;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ProjectManifest {
    DoveToml(ManifestPath),
    // MoveToml(ManifestPath),
}

impl ProjectManifest {
    pub fn from_manifest_file(path: AbsPathBuf) -> Result<ProjectManifest> {
        let path = ManifestPath::try_from(path)
            .map_err(|path| format_err!("bad manifest path: {}", path.display()))?;
        // if path.file_name().unwrap_or_default() == "Move.toml" {
        //     return Ok(ProjectManifest::MoveToml(path));
        // }
        if path.file_name().unwrap_or_default() == "Dove.toml" {
            return Ok(ProjectManifest::DoveToml(path));
        }
        bail!(
            "project root must point to Dove.toml (dove) or Move.toml (move-cli): {}",
            path.display()
        )
    }

    pub fn discover_single(path: &AbsPath) -> Result<ProjectManifest> {
        let mut candidates = ProjectManifest::discover(path)?;
        let res = match candidates.pop() {
            None => bail!("no projects"),
            Some(it) => it,
        };

        if !candidates.is_empty() {
            bail!("more than one project")
        }
        Ok(res)
    }

    pub fn discover(path: &AbsPath) -> io::Result<Vec<ProjectManifest>> {
        // if let Some(project_json) = find_in_parent_dirs(path, "rust-project.json") {
        //     return Ok(vec![ProjectManifest::ProjectJson(project_json)]);
        // }
        return find_dove_toml(path)
            .map(|paths| paths.into_iter().map(ProjectManifest::DoveToml).collect());

        fn find_dove_toml(path: &AbsPath) -> io::Result<Vec<ManifestPath>> {
            match find_in_parent_dirs(path, "Dove.toml") {
                Some(it) => Ok(vec![it]),
                None => Ok(find_cargo_toml_in_child_dir(read_dir(path)?)),
            }
        }

        fn find_in_parent_dirs(path: &AbsPath, target_file_name: &str) -> Option<ManifestPath> {
            if path.file_name().unwrap_or_default() == target_file_name {
                if let Ok(manifest) = ManifestPath::try_from(path.to_path_buf()) {
                    return Some(manifest);
                }
            }

            let mut curr = Some(path);

            while let Some(path) = curr {
                let candidate = path.join(target_file_name);
                if fs::metadata(&candidate).is_ok() {
                    if let Ok(manifest) = ManifestPath::try_from(candidate) {
                        return Some(manifest);
                    }
                }
                curr = path.parent();
            }

            None
        }

        fn find_cargo_toml_in_child_dir(entities: ReadDir) -> Vec<ManifestPath> {
            // Only one level down to avoid cycles the easy way and stop a runaway scan with large projects
            entities
                .filter_map(Result::ok)
                .map(|it| it.path().join("Dove.toml"))
                .filter(|it| it.exists())
                .map(AbsPathBuf::assert)
                .filter_map(|it| it.try_into().ok())
                .collect()
        }
    }

    pub fn discover_all(paths: &[AbsPathBuf]) -> Vec<ProjectManifest> {
        let mut res = paths
            .iter()
            .filter_map(|it| ProjectManifest::discover(it.as_ref()).ok())
            .flatten()
            .collect::<FxHashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        res.sort();
        res
    }
}

fn utf8_stdout(mut cmd: Command) -> Result<String> {
    let output = cmd.output().with_context(|| format!("{:?} failed", cmd))?;
    if !output.status.success() {
        match String::from_utf8(output.stderr) {
            Ok(stderr) if !stderr.is_empty() => {
                bail!("{:?} failed, {}\nstderr:\n{}", cmd, output.status, stderr)
            }
            _ => bail!("{:?} failed, {}", cmd, output.status),
        }
    }
    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_string())
}
