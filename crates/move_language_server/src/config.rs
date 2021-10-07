use crate::line_index::OffsetEncoding;
use crate::lsp_ext::supports_utf8;
use project_model::ProjectManifest;
use vfs::AbsPathBuf;

#[derive(Debug, Clone)]
pub struct FilesConfig {
    pub exclude: Vec<AbsPathBuf>,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub caps: lsp_types::ClientCapabilities,
    pub discovered_projects: Option<Vec<ProjectManifest>>,
    detached_files: Vec<AbsPathBuf>,
    pub root_path: AbsPathBuf,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LinkedProject {
    pub manifest: ProjectManifest,
}

impl From<ProjectManifest> for LinkedProject {
    fn from(v: ProjectManifest) -> Self {
        LinkedProject { manifest: v }
    }
}

macro_rules! try_ {
    ($expr:expr) => {
        || -> _ { Some($expr) }()
    };
}
macro_rules! try_or {
    ($expr:expr, $or:expr) => {
        try_!($expr).unwrap_or($or)
    };
}

impl Config {
    pub fn new(root_path: AbsPathBuf, caps: lsp_types::ClientCapabilities) -> Self {
        Config { caps, discovered_projects: None, root_path, detached_files: vec![] }
    }

    pub fn linked_projects(&self) -> Vec<LinkedProject> {
        self.discovered_projects
            .as_ref()
            .into_iter()
            .flatten()
            .cloned()
            .map(LinkedProject::from)
            .collect()
    }

    pub fn detached_files(&self) -> &[AbsPathBuf] {
        &self.detached_files
    }

    pub fn did_save_text_document_dynamic_registration(&self) -> bool {
        let caps =
            try_or!(self.caps.text_document.as_ref()?.synchronization.clone()?, Default::default());
        caps.did_save == Some(true) && caps.dynamic_registration == Some(true)
    }

    pub fn did_change_watched_files_dynamic_registration(&self) -> bool {
        try_or!(
            self.caps.workspace.as_ref()?.did_change_watched_files.as_ref()?.dynamic_registration?,
            false
        )
    }

    pub fn offset_encoding(&self) -> OffsetEncoding {
        if supports_utf8(&self.caps) {
            OffsetEncoding::Utf8
        } else {
            OffsetEncoding::Utf16
        }
    }

    // pub fn files(&self) -> FilesConfig {
    //     FilesConfig {
    //         exclude: self.data.files_excludeDirs.iter().map(|it| self.root_path.join(it)).collect(),
    //     }
    // }
}
