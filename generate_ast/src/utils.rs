use xshell::{cmd, pushenv};

pub(crate) fn pluralize(s: &str) -> String {
    format!("{}s", s)
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
