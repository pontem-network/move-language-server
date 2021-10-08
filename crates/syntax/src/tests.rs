use crate::ast::SourceFile;
use crate::syntax_error::SyntaxError;
use expect_test::expect_file;
use std::fs;
use std::path::{Path, PathBuf};
use test_utils::project_root;

#[test]
fn parser_tests() {
    dir_tests(&test_data_dir(), &["parser/ok"], "txt", |text, path| {
        let parse = SourceFile::parse(text);
        let errors = parse.errors();
        assert_errors_are_absent(errors, path);
        parse.debug_dump()
    });
    dir_tests(&test_data_dir(), &["parser/err"], "txt", |text, path| {
        let parse = SourceFile::parse(text);
        let errors = parse.errors();
        assert_errors_are_present(errors, path);
        parse.debug_dump()
    });
}

pub fn test_data_dir() -> PathBuf {
    project_root().join("crates/syntax/test_data")
}

fn assert_errors_are_present(errors: &[SyntaxError], path: &Path) {
    assert!(!errors.is_empty(), "There should be errors in the file {:?}", path.display());
}
fn assert_errors_are_absent(errors: &[SyntaxError], path: &Path) {
    assert_eq!(
        errors,
        &[] as &[SyntaxError],
        "There should be no errors in the file {:?}",
        path.display(),
    );
}

/// Calls callback `f` with input code and file paths for each `.move` file in `test_data_dir`
/// subdirectories defined by `paths`.
///
/// If the content of the matching output file differs from the output of `f()`
/// the test will fail.
///
/// If there is no matching output file it will be created and filled with the
/// output of `f()`, but the test will fail.
fn dir_tests<F>(test_data_dir: &Path, paths: &[&str], outfile_extension: &str, f: F)
where
    F: Fn(&str, &Path) -> String,
{
    for (path, input_code) in collect_move_files(test_data_dir, paths) {
        let actual = f(&input_code, &path);
        let path = path.with_extension(outfile_extension);
        expect_file![path].assert_eq(&actual)
    }
}

/// Collects all `.move` files from `dir` subdirectories defined by `paths`.
fn collect_move_files(root_dir: &Path, paths: &[&str]) -> Vec<(PathBuf, String)> {
    paths
        .iter()
        .flat_map(|path| {
            let path = root_dir.to_owned().join(path);
            move_files_in_dir(&path).into_iter()
        })
        .map(|path| {
            let text = read_text(&path);
            (path, text)
        })
        .collect()
}

/// Collects paths to all `.move` files from `dir` in a sorted `Vec<PathBuf>`.
fn move_files_in_dir(dir: &Path) -> Vec<PathBuf> {
    let mut acc = Vec::new();
    for file in fs::read_dir(&dir).unwrap() {
        let file = file.unwrap();
        let path = file.path();
        if path.extension().unwrap_or_default() == "move" {
            acc.push(path);
        }
    }
    acc.sort();
    acc
}

/// Read file and normalize newlines.
///
/// `rustc` seems to always normalize `\r\n` newlines to `\n`:
///
/// ```
/// let s = "
/// ";
/// assert_eq!(s.as_bytes(), &[10]);
/// ```
///
/// so this should always be correct.
fn read_text(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("File at {:?} should be valid", path))
        .replace("\r\n", "\n")
}
