//! Project loading & configuration updates
use ide_db::base_db::{SourceRoot, VfsPath};
use project_model::ProjectWorkspace;
use std::{mem, sync::Arc};

use crate::config::LinkedProject;
use crate::global_state::GlobalState;
use crate::lsp_ext;
use vfs::{file_set::FileSetConfig, AbsPath, AbsPathBuf, ChangeKind};

// #[derive(Debug)]
// pub(crate) enum ProjectWorkspaceProgress {
//     Begin,
//     Report(String),
//     End(Vec<anyhow::Result<ProjectWorkspace>>),
// }

impl GlobalState {
    pub(crate) fn is_quiescent(&self) -> bool {
        !(self.vfs_progress_config_version < self.vfs_config_version
            || self.vfs_progress_n_done < self.vfs_progress_n_total)
    }

    pub(crate) fn current_status(&self) -> lsp_ext::ServerStatusParams {
        let mut status = lsp_ext::ServerStatusParams {
            health: lsp_ext::Health::Ok,
            quiescent: self.is_quiescent(),
            message: None,
        };
        status
    }

    pub(crate) fn fetch_workspaces(&mut self) {
        tracing::info!("will fetch workspaces");

        self.task_pool.handle.spawn_with_sender({
            let linked_projects = self.config.linked_projects();
            let detached_files = self.config.detached_files().to_vec();
            move |sender| {
                // let progress = {
                //     let sender = sender.clone();
                //     move |msg| {
                //         sender
                //             .send(Task::FetchWorkspace(ProjectWorkspaceProgress::Report(msg)))
                //             .unwrap()
                //     }
                // };
                //
                // sender.send(Task::FetchWorkspace(ProjectWorkspaceProgress::Begin)).unwrap();

                let mut workspaces = linked_projects
                    .iter()
                    .map(|project| ProjectWorkspace::load(project.manifest.clone()))
                    .collect::<Vec<_>>();
                if !detached_files.is_empty() {
                    workspaces.push(ProjectWorkspace::load_detached_files(detached_files));
                }

                // tracing::info!("did fetch workspaces {:?}", workspaces);
                // sender
                //     .send(Task::FetchWorkspace(ProjectWorkspaceProgress::End(workspaces)))
                //     .unwrap();
            }
        });
    }
}

#[derive(Default, Debug)]
pub struct SourceRootConfig {
    pub(crate) fsc: FileSetConfig,
    pub(crate) local_filesets: Vec<usize>,
}

impl SourceRootConfig {
    pub fn partition(&self, vfs: &vfs::Vfs) -> Vec<SourceRoot> {
        self.fsc
            .partition(vfs)
            .into_iter()
            .enumerate()
            .map(|(idx, file_set)| {
                let is_local = self.local_filesets.contains(&idx);
                if is_local {
                    SourceRoot::new_local(file_set)
                } else {
                    SourceRoot::new_library(file_set)
                }
            })
            .collect()
    }
}

pub(crate) fn should_refresh_for_change(path: &AbsPath, change_kind: ChangeKind) -> bool {
    const IMPLICIT_TARGET_FILES: &[&str] = &["build.rs", "src/main.rs", "src/lib.rs"];
    const IMPLICIT_TARGET_DIRS: &[&str] = &["src/bin", "examples", "tests", "benches"];
    let file_name = path.file_name().unwrap_or_default();

    if file_name == "Cargo.toml" || file_name == "Cargo.lock" {
        return true;
    }
    if change_kind == ChangeKind::Modify {
        return false;
    }
    if path.extension().unwrap_or_default() != "rs" {
        return false;
    }
    if IMPLICIT_TARGET_FILES.iter().any(|it| path.as_ref().ends_with(it)) {
        return true;
    }
    let parent = match path.parent() {
        Some(it) => it,
        None => return false,
    };
    if IMPLICIT_TARGET_DIRS.iter().any(|it| parent.as_ref().ends_with(it)) {
        return true;
    }
    if file_name == "main.rs" {
        let grand_parent = match parent.parent() {
            Some(it) => it,
            None => return false,
        };
        if IMPLICIT_TARGET_DIRS.iter().any(|it| grand_parent.as_ref().ends_with(it)) {
            return true;
        }
    }
    false
}
