//! base_db defines basic database traits. The concrete DB is defined by ide.
mod input;
mod change;
pub mod fixture;

use std::{panic, sync::Arc};

use syntax::{ast, Parse, SourceFile, TextRange, TextSize};

use crate::input::{SourceRoot, SourceRootId};
pub use salsa::{self, Cancelled};
pub use vfs::{file_set::FileSet, AnchoredPath, AnchoredPathBuf, FileId, VfsPath};

#[macro_export]
macro_rules! impl_intern_key {
    ($name:ident) => {
        impl $crate::salsa::InternKey for $name {
            fn from_intern_id(v: $crate::salsa::InternId) -> Self {
                $name(v)
            }
            fn as_intern_id(&self) -> $crate::salsa::InternId {
                self.0
            }
        }
    };
}

pub trait Upcast<T: ?Sized> {
    fn upcast(&self) -> &T;
}

#[derive(Clone, Copy, Debug)]
pub struct FilePosition {
    pub file_id: FileId,
    pub offset: TextSize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct FileRange {
    pub file_id: FileId,
    pub range: TextRange,
}

/// Database which stores all significant input facts: source code and project
/// model. Everything else in rust-analyzer is derived from these queries.
#[salsa::query_group(SourceDatabaseStorage)]
pub trait SourceDatabase: std::fmt::Debug {
    // Parses the file into the syntax tree.
    #[salsa::invoke(parse_query)]
    fn parse(&self, file_id: FileId) -> Parse<ast::SourceFile>;

    #[salsa::input]
    fn file_text(&self, file_id: FileId) -> Arc<String>;

    /// Path to a file, relative to the root of its source root.
    /// Source root of the file.
    #[salsa::input]
    fn file_source_root(&self, file_id: FileId) -> SourceRootId;

    /// Contents of the source root.
    #[salsa::input]
    fn source_root(&self, id: SourceRootId) -> Arc<SourceRoot>;
}

fn parse_query(db: &dyn SourceDatabase, file_id: FileId) -> Parse<ast::SourceFile> {
    let text = db.file_text(file_id);
    SourceFile::parse(&*text)
}
