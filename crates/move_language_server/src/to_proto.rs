//! Conversion of rust-analyzer specific types to lsp_types equivalents.
use lsp_types::TextEdit;
use std::{
    iter::once,
    path,
    sync::atomic::{AtomicU32, Ordering},
};

use crate::global_state::GlobalStateSnapshot;
use crate::line_index::{LineEndings, LineIndex, OffsetEncoding};
use crate::Result;
use ide_db::base_db::{FileId, FileRange};
use ide_diagnostics::Severity;
use itertools::Itertools;
use serde_json::to_value;
use syntax::{TextRange, TextSize};
use vfs::AbsPath;

pub(crate) fn position(line_index: &LineIndex, offset: TextSize) -> lsp_types::Position {
    let line_col = line_index.index.line_col(offset);
    match line_index.encoding {
        OffsetEncoding::Utf8 => lsp_types::Position::new(line_col.line, line_col.col),
        OffsetEncoding::Utf16 => {
            let line_col = line_index.index.to_utf16(line_col);
            lsp_types::Position::new(line_col.line, line_col.col)
        }
    }
}

pub(crate) fn range(line_index: &LineIndex, range: TextRange) -> lsp_types::Range {
    let start = position(line_index, range.start());
    let end = position(line_index, range.end());
    lsp_types::Range::new(start, end)
}

pub(crate) fn diagnostic_severity(severity: Severity) -> lsp_types::DiagnosticSeverity {
    match severity {
        Severity::Error => lsp_types::DiagnosticSeverity::Error,
        Severity::WeakWarning => lsp_types::DiagnosticSeverity::Hint,
    }
}

pub(crate) fn url(snap: &GlobalStateSnapshot, file_id: FileId) -> lsp_types::Url {
    snap.file_id_to_url(file_id)
}

/// Returns a `Url` object from a given path, will lowercase drive letters if present.
/// This will only happen when processing windows paths.
///
/// When processing non-windows path, this is essentially the same as `Url::from_file_path`.
pub(crate) fn url_from_abs_path(path: &AbsPath) -> lsp_types::Url {
    let url = lsp_types::Url::from_file_path(path).unwrap();
    match path.as_ref().components().next() {
        Some(path::Component::Prefix(prefix))
            if matches!(prefix.kind(), path::Prefix::Disk(_) | path::Prefix::VerbatimDisk(_)) =>
        {
            // Need to lowercase driver letter
        }
        _ => return url,
    }

    let driver_letter_range = {
        let (scheme, drive_letter, _rest) = match url.as_str().splitn(3, ':').collect_tuple() {
            Some(it) => it,
            None => return url,
        };
        let start = scheme.len() + ':'.len_utf8();
        start..(start + drive_letter.len())
    };

    // Note: lowercasing the `path` itself doesn't help, the `Url::parse`
    // machinery *also* canonicalizes the drive letter. So, just massage the
    // string in place.
    let mut url: String = url.into();
    url[driver_letter_range].make_ascii_lowercase();
    lsp_types::Url::parse(&url).unwrap()
}

pub(crate) fn optional_versioned_text_document_identifier(
    snap: &GlobalStateSnapshot,
    file_id: FileId,
) -> lsp_types::OptionalVersionedTextDocumentIdentifier {
    let url = url(snap, file_id);
    let version = snap.url_file_version(&url);
    lsp_types::OptionalVersionedTextDocumentIdentifier { uri: url, version }
}

pub(crate) fn location(
    snap: &GlobalStateSnapshot,
    frange: FileRange,
) -> Result<lsp_types::Location> {
    let url = url(snap, frange.file_id);
    let line_index = snap.file_line_index(frange.file_id)?;
    let range = range(&line_index, frange.range);
    let loc = lsp_types::Location::new(url, range);
    Ok(loc)
}

pub(crate) fn implementation_title(count: usize) -> String {
    if count == 1 {
        "1 implementation".into()
    } else {
        format!("{} implementations", count)
    }
}

pub(crate) fn reference_title(count: usize) -> String {
    if count == 1 {
        "1 reference".into()
    } else {
        format!("{} references", count)
    }
}
