use crate::{to_proto, Result};

use crate::global_state::GlobalStateSnapshot;
use lsp_types::{Diagnostic, DiagnosticTag, NumberOrString};
use vfs::FileId;

pub(crate) fn publish_diagnostics(
    snap: &GlobalStateSnapshot,
    file_id: FileId,
) -> Result<Vec<Diagnostic>> {
    let line_index = snap.file_line_index(file_id)?;

    let diagnostics: Vec<Diagnostic> = snap
        .analysis
        .diagnostics(file_id)?
        .into_iter()
        .map(|d| Diagnostic {
            range: to_proto::range(&line_index, d.range),
            severity: Some(to_proto::diagnostic_severity(d.severity)),
            code: Some(NumberOrString::String(d.code.as_str().to_string())),
            code_description: Some(lsp_types::CodeDescription {
                href: lsp_types::Url::parse(&format!(
                    "https://rust-analyzer.github.io/manual.html#{}",
                    d.code.as_str()
                ))
                .unwrap(),
            }),
            source: Some("move-language-server".to_string()),
            message: d.message,
            related_information: None,
            tags: if d.unused { Some(vec![DiagnosticTag::Unnecessary]) } else { None },
            data: None,
        })
        .collect();
    Ok(diagnostics)
}
