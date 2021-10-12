#![allow(unused)]

use std::marker::PhantomData;
use std::sync::Arc;

use stdx::format_to;

/// `SourceFile` represents a parse tree for a single Rust file.
use rowan::GreenNode;

use crate::syntax_error::SyntaxError;

pub mod ast;
pub mod parsing;
pub mod syntax_error;
pub mod syntax_node;
mod tests;

pub use crate::ast::{AstNode, SourceFile};
use crate::syntax_node::SyntaxNode;
pub use parser::SyntaxKind;
pub use smol_str::SmolStr;
pub use text_size::{TextRange, TextSize};

/// `Parse` is the result of the parsing: a syntax tree and a collection of
/// errors.
///
/// Note that we always produce a syntax tree, even for completely invalid
/// files.
#[derive(Debug, PartialEq, Eq)]
pub struct Parse<T> {
    green: GreenNode,
    errors: Arc<Vec<SyntaxError>>,
    _ty: PhantomData<fn() -> T>,
}

impl<T> Clone for Parse<T> {
    fn clone(&self) -> Parse<T> {
        Parse { green: self.green.clone(), errors: self.errors.clone(), _ty: PhantomData }
    }
}

impl<T> Parse<T> {
    fn new(green: GreenNode, errors: Vec<SyntaxError>) -> Parse<T> {
        Parse { green, errors: Arc::new(errors), _ty: PhantomData }
    }

    pub fn syntax_node(&self) -> SyntaxNode {
        SyntaxNode::new_root(self.green.clone())
    }
}

impl<T: AstNode> Parse<T> {
    pub fn to_syntax(self) -> Parse<SyntaxNode> {
        Parse { green: self.green, errors: self.errors, _ty: PhantomData }
    }

    pub fn tree(&self) -> T {
        T::cast(self.syntax_node()).unwrap()
    }

    pub fn errors(&self) -> &[SyntaxError] {
        &*self.errors
    }

    pub fn ok(self) -> Result<T, Arc<Vec<SyntaxError>>> {
        if self.errors.is_empty() {
            Ok(self.tree())
        } else {
            Err(self.errors)
        }
    }
}

impl Parse<SyntaxNode> {
    pub fn cast<N: AstNode>(self) -> Option<Parse<N>> {
        if N::cast(self.syntax_node()).is_some() {
            Some(Parse { green: self.green, errors: self.errors, _ty: PhantomData })
        } else {
            None
        }
    }
}

impl Parse<SourceFile> {
    pub fn debug_dump(&self) -> String {
        let mut buf = format!("{:#?}", self.tree().syntax());
        for err in self.errors.iter() {
            format_to!(buf, "error {:?}: {}\n", err.range(), err);
        }
        buf
    }

    // pub fn reparse(&self, indel: &Indel) -> Parse<SourceFile> {
    //     self.incremental_reparse(indel).unwrap_or_else(|| self.full_reparse(indel))
    // }

    // fn incremental_reparse(&self, indel: &Indel) -> Option<Parse<SourceFile>> {
    //     // FIXME: validation errors are not handled here
    //     parsing::incremental_reparse(self.tree().syntax(), indel, self.errors.to_vec()).map(
    //         |(green_node, errors, _reparsed_range)| Parse {
    //             green: green_node,
    //             errors: Arc::new(errors),
    //             _ty: PhantomData,
    //         },
    //     )
    // }

    // fn full_reparse(&self, indel: &Indel) -> Parse<SourceFile> {
    //     let mut text = self.tree().syntax().text().to_string();
    //     indel.apply(&mut text);
    //     SourceFile::parse(&text)
    // }
}

impl SourceFile {
    pub fn parse(text: &str) -> Parse<SourceFile> {
        let (green, mut errors) = parsing::parse_text(text);
        let root = SyntaxNode::new_root(green.clone());

        // if cfg!(debug_assertions) {
        //     validation::validate_block_structure(&root);
        // }

        // errors.extend(validation::validate(&root));

        assert_eq!(root.kind(), SyntaxKind::SOURCE_FILE);
        Parse { green, errors: Arc::new(errors), _ty: PhantomData }
    }
}

#[cfg(test)]
mod inline_tests {
    use super::*;
    use crate::tests::test_data_dir;
    use std::fs::read_to_string;
    use std::path::Path;
    use test_utils::project_root;

    #[test]
    fn parser_test() {
        let fname = "ok/address_definitions.move";
        let file = test_data_dir().join("parser").join(fname);
        let text = read_to_string(test_data_dir().join("parser").join(fname)).unwrap();

        let (green, mut errors) = parsing::parse_text(&text);
        let root = SyntaxNode::new_root(green.clone());

        // let source_file = SourceFile { syntax: root };
        // let parse_tree = SourceFile::parse(text);
        // dbg!(&parse_tree);
        // let (green, mut errors) = parsing::parse_text(text);
        // let root = SyntaxNode::new_root(green.clone());
        dbg!(&errors);
        dbg!(&root);
    }
}
