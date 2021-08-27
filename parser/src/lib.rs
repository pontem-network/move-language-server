#![allow(unused)]

use crate::event::Event;
pub use lexer::{Lexer, Token};
pub use syntax_kind::SyntaxKind;
pub use token_set::TokenSet;

#[macro_use]
mod token_set;
#[macro_use]
pub mod syntax_kind;

pub mod event;
mod grammar;
mod lexer;
mod marker;
mod parser;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseError(pub Box<String>);

/// `TreeSink` abstracts details of a particular syntax tree implementation.
pub trait TreeSink {
    /// Adds new token to the current branch.
    fn token(&mut self, kind: SyntaxKind, n_tokens: u8);

    /// Start new branch and make it current.
    fn start_node(&mut self, kind: SyntaxKind);

    /// Finish current branch and restore previous
    /// branch as current.
    fn finish_node(&mut self);

    fn error(&mut self, error: ParseError);
}

pub fn parse_with_lexer<'t>(lexer: &'t mut Lexer<'t>) -> (Vec<Event>, Vec<Token>) {
    let mut p = parser::Parser::new(lexer);
    grammar::root(&mut p);
    let tokens = p.tokens();
    let events = p.finish();
    (events, tokens)
}

fn parse_from_tokens<'t, F>(lexer: &'t mut Lexer<'t>, tree_sink: &mut dyn TreeSink, f: F)
where
    F: FnOnce(&mut parser::Parser),
{
    let mut p = parser::Parser::new(lexer);
    f(&mut p);
    let events = p.finish();
    event::process(tree_sink, events);
}

/// Parse given tokens into the given sink as a rust file.
pub fn parse<'t>(token_source: &'t mut Lexer<'t>, tree_sink: &mut dyn TreeSink) {
    parse_from_tokens(token_source, tree_sink, grammar::root);
}
