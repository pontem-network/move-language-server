//! Defines `Fixture` -- a convenient way to describe the initial state of
//! rust-analyzer database from a single string.
//!
//! Fixtures are strings containing rust source code with optional metadata.
//! A fixture without metadata is parsed into a single source file.
//! Use this to test functionality local to one file.
//!
//! Simple Example:
//! ```
//! r#"
//! fn main() {
//!     println!("Hello World")
//! }
//! "#
//! ```
//!
//! Metadata can be added to a fixture after a `//-` comment.
//! The basic form is specifying filenames,
//! which is also how to define multiple files in a single test fixture
//!
//! Example using two files in the same crate:
//! ```
//! "
//! //- /main.rs
//! mod foo;
//! fn main() {
//!     foo::bar();
//! }
//!
//! //- /foo.rs
//! pub fn bar() {}
//! "
//! ```
//!
//! Example using two crates with one file each, with one crate depending on the other:
//! ```
//! r#"
//! //- /main.rs crate:a deps:b
//! fn main() {
//!     b::foo();
//! }
//! //- /lib.rs crate:b
//! pub fn b() {
//!     println!("Hello World")
//! }
//! "#
//! ```
//!
//! Metadata allows specifying all settings and variables
//! that are available in a real rust project:
//! - crate names via `crate:cratename`
//! - dependencies via `deps:dep1,dep2`
//! - configuration settings via `cfg:dbg=false,opt_level=2`
//! - environment variables via `env:PATH=/bin,RUST_LOG=debug`
//!
//! Example using all available metadata:
//! ```
//! "
//! //- /lib.rs crate:foo deps:bar,baz cfg:foo=a,bar=b env:OUTDIR=path/to,OTHER=foo
//! fn insert_source_code_here() {}
//! "
//! ```

use rustc_hash::FxHashMap;
use stdx::trim_indent;

#[derive(Debug, Eq, PartialEq)]
pub struct Fixture {
    pub path: String,
    pub text: String,
    pub krate: Option<String>,
    pub deps: Vec<String>,
    pub extern_prelude: Option<Vec<String>>,
    pub cfg_atoms: Vec<String>,
    pub cfg_key_values: Vec<(String, String)>,
    pub edition: Option<String>,
    pub env: FxHashMap<String, String>,
    pub introduce_new_source_root: Option<String>,
}

impl Fixture {
    /// Parses text which looks like this:
    ///
    ///  ```not_rust
    ///  //- some meta
    ///  line 1
    ///  line 2
    ///  //- other meta
    ///  ```
    ///
    /// Fixture can also start with a proc_macros and minicore declaration(in that order):
    ///
    /// ```
    /// //- proc_macros: identity
    /// //- minicore: sized
    /// ```
    ///
    /// That will include predefined proc macros and a subset of `libcore` into the fixture, see
    /// `minicore.rs` for what's available.
    pub fn parse(ra_fixture: &str) -> Vec<Fixture> {
        let fixture = trim_indent(ra_fixture);
        let fixture = fixture.as_str();
        let mut res: Vec<Fixture> = Vec::new();

        let default = if fixture.contains("//-") { None } else { Some("//- /main.move") };

        for (ix, line) in default.into_iter().chain(fixture.split_inclusive('\n')).enumerate() {
            if line.contains("//-") {
                assert!(
                    line.starts_with("//-"),
                    "Metadata line {} has invalid indentation. \
                     All metadata lines need to have the same indentation.\n\
                     The offending line: {:?}",
                    ix,
                    line
                );
            }

            // if line.starts_with("//-") {
            //     let meta = Fixture::parse_meta_line(line);
            //     res.push(meta)
            // } else {
            if line.starts_with("// ")
                && line.contains(':')
                && !line.contains("::")
                && line.chars().all(|it| !it.is_uppercase())
            {
                panic!("looks like invalid metadata line: {:?}", line)
            }

            if let Some(entry) = res.last_mut() {
                entry.text.push_str(line);
            }
            // }
        }

        res
    }
}

#[test]
#[should_panic]
fn parse_fixture_checks_further_indented_metadata() {
    Fixture::parse(
        r"
        //- /lib.rs
          mod bar;

          fn foo() {}
          //- /bar.rs
          pub fn baz() {}
          ",
    );
}

#[test]
fn parse_fixture_gets_full_meta() {
    let parsed = Fixture::parse(
        r#"
//- proc_macros: identity
//- minicore: coerce_unsized
//- /lib.rs crate:foo deps:bar,baz cfg:foo=a,bar=b,atom env:OUTDIR=path/to,OTHER=foo
mod m;
"#,
    );
    // assert_eq!(proc_macros, vec!["identity".to_string()]);
    // assert_eq!(mini_core.unwrap().activated_flags, vec!["coerce_unsized".to_string()]);
    assert_eq!(1, parsed.len());

    let meta = &parsed[0];
    assert_eq!("mod m;\n", meta.text);

    assert_eq!("foo", meta.krate.as_ref().unwrap());
    assert_eq!("/lib.rs", meta.path);
    assert_eq!(2, meta.env.len());
}
