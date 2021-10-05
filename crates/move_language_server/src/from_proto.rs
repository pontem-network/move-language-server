//! Conversion lsp_types types to rust-analyzer specific ones.
use std::convert::TryFrom;

use ide_db::base_db::{FileId, FilePosition, FileRange};
use ide_db::{LineCol, LineColUtf16};
use syntax::{TextRange, TextSize};
use vfs::AbsPathBuf;

use crate::{
    from_json,
    global_state::GlobalStateSnapshot,
    line_index::{LineIndex, OffsetEncoding},
    lsp_utils::invalid_params_error,
    Result,
};

pub(crate) fn abs_path(url: &lsp_types::Url) -> Result<AbsPathBuf> {
    let path = url.to_file_path().map_err(|()| "url is not a file")?;
    Ok(AbsPathBuf::try_from(path).unwrap())
}

pub(crate) fn vfs_path(url: &lsp_types::Url) -> Result<vfs::VfsPath> {
    abs_path(url).map(vfs::VfsPath::from)
}

pub(crate) fn offset(line_index: &LineIndex, position: lsp_types::Position) -> TextSize {
    let line_col = match line_index.encoding {
        OffsetEncoding::Utf8 => {
            LineCol { line: position.line as u32, col: position.character as u32 }
        }
        OffsetEncoding::Utf16 => {
            let line_col =
                LineColUtf16 { line: position.line as u32, col: position.character as u32 };
            line_index.index.to_utf8(line_col)
        }
    };
    line_index.index.offset(line_col)
}

pub(crate) fn text_range(line_index: &LineIndex, range: lsp_types::Range) -> TextRange {
    let start = offset(line_index, range.start);
    let end = offset(line_index, range.end);
    TextRange::new(start, end)
}

pub(crate) fn file_id(snap: &GlobalStateSnapshot, url: &lsp_types::Url) -> Result<FileId> {
    snap.url_to_file_id(url)
}

pub(crate) fn file_position(
    snap: &GlobalStateSnapshot,
    tdpp: lsp_types::TextDocumentPositionParams,
) -> Result<FilePosition> {
    let file_id = file_id(snap, &tdpp.text_document.uri)?;
    let line_index = snap.file_line_index(file_id)?;
    let offset = offset(&line_index, tdpp.position);
    Ok(FilePosition { file_id, offset })
}

pub(crate) fn file_range(
    snap: &GlobalStateSnapshot,
    text_document_identifier: lsp_types::TextDocumentIdentifier,
    range: lsp_types::Range,
) -> Result<FileRange> {
    let file_id = file_id(snap, &text_document_identifier.uri)?;
    let line_index = snap.file_line_index(file_id)?;
    let range = text_range(&line_index, range);
    Ok(FileRange { file_id, range })
}
