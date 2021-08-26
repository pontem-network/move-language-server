use std::fs;
use std::path::{Path, PathBuf};
use xshell::{cmd, pushenv};

pub(crate) fn to_upper_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_uppercase());
    }
    buf
}

pub(crate) fn to_lower_snake_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev = false;
    for c in s.chars() {
        if c.is_ascii_uppercase() && prev {
            buf.push('_')
        }
        prev = true;

        buf.push(c.to_ascii_lowercase());
    }
    buf
}

pub(crate) fn to_pascal_case(s: &str) -> String {
    let mut buf = String::with_capacity(s.len());
    let mut prev_is_underscore = true;
    for c in s.chars() {
        if c == '_' {
            prev_is_underscore = true;
        } else if prev_is_underscore {
            buf.push(c.to_ascii_uppercase());
            prev_is_underscore = false;
        } else {
            buf.push(c.to_ascii_lowercase());
        }
    }
    buf
}

pub(crate) fn pluralize(s: &str) -> String {
    format!("{}s", s)
}

pub(crate) fn cargo_project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Checks that the `file` has the specified `contents`. If that is not the
/// case, updates the file and then fails the test.
pub(crate) fn ensure_file_contents(file: &Path, contents: &str) {
    if let Ok(old_contents) = fs::read_to_string(file) {
        if normalize_newlines(&old_contents) == normalize_newlines(contents) {
            // File is already up to date.
            return;
        }
    }

    let display_path = file.strip_prefix(&cargo_project_root()).unwrap_or(file);
    eprintln!(
        "\n\x1b[31;1merror\x1b[0m: {} was not up-to-date, updating\n",
        display_path.display()
    );
    if std::env::var("CI").is_ok() {
        eprintln!("    NOTE: run `cargo test` locally and commit the updated files\n");
    }
    if let Some(parent) = file.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(file, contents).unwrap();
    panic!("some file was not up to date and has been updated, simply re-run the tests")
}

pub(crate) fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

pub(crate) fn ensure_rustfmt() {
    let version = cmd!("rustfmt --version").read().unwrap_or_default();
    if !version.contains("stable") {
        panic!(
            "Failed to run rustfmt from toolchain 'stable'. \
                 Please run `rustup component add rustfmt --toolchain stable` to install it.",
        )
    }
}

pub(crate) fn reformat(text: String) -> String {
    let _e = pushenv("RUSTUP_TOOLCHAIN", "stable");
    ensure_rustfmt();
    let mut stdout = cmd!("rustfmt --config fn_single_line=true")
        .stdin(text)
        .read()
        .unwrap();
    if !stdout.ends_with('\n') {
        stdout.push('\n');
    }
    stdout
}
