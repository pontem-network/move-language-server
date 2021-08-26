#[macro_use]
mod token_set;
#[macro_use]
pub mod syntax_kind;

mod event;
mod grammar;
mod lexer;
mod parser;

use crate::lexer::Lexer;
pub use lexer::Token;
pub use syntax_kind::SyntaxKind;
pub use token_set::TokenSet;

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

// #[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
// pub enum FragmentKind {
//     Path,
//     Expr,
//     Statement,
//     StatementOptionalSemi,
//     Type,
//     Pattern,
//     Item,
//     Block,
//     Visibility,
//     MetaItem,
//
//     // These kinds are used when parsing the result of expansion
//     // FIXME: use separate fragment kinds for macro inputs and outputs?
//     Items,
//     Statements,
//
//     Attr,
// }
//
// pub fn parse_fragment(
//     token_source: &mut dyn TokenSource,
//     tree_sink: &mut dyn TreeSink,
//     fragment_kind: FragmentKind,
// ) {
//     let parser: fn(&'_ mut parser::Parser) = match fragment_kind {
//         FragmentKind::Path => grammar::fragments::path,
//         FragmentKind::Expr => grammar::fragments::expr,
//         FragmentKind::Type => grammar::fragments::type_,
//         FragmentKind::Pattern => grammar::fragments::pattern_single,
//         FragmentKind::Item => grammar::fragments::item,
//         FragmentKind::Block => grammar::fragments::block_expr,
//         FragmentKind::Visibility => grammar::fragments::opt_visibility,
//         FragmentKind::MetaItem => grammar::fragments::meta_item,
//         FragmentKind::Statement => grammar::fragments::stmt,
//         FragmentKind::StatementOptionalSemi => grammar::fragments::stmt_optional_semi,
//         FragmentKind::Items => grammar::fragments::macro_items,
//         FragmentKind::Statements => grammar::fragments::macro_stmts,
//         FragmentKind::Attr => grammar::fragments::attr,
//     };
//     parse_from_tokens(token_source, tree_sink, parser)
// }
//
// /// A parsing function for a specific braced-block.
// pub struct Reparser(fn(&mut parser::Parser));
//
// impl Reparser {
//     /// If the node is a braced block, return the corresponding `Reparser`.
//     pub fn for_node(
//         node: SyntaxKind,
//         first_child: Option<SyntaxKind>,
//         parent: Option<SyntaxKind>,
//     ) -> Option<Reparser> {
//         grammar::reparser(node, first_child, parent).map(Reparser)
//     }
//
//     /// Re-parse given tokens using this `Reparser`.
//     ///
//     /// Tokens must start with `{`, end with `}` and form a valid brace
//     /// sequence.
//     pub fn parse(self, token_source: &mut dyn TokenSource, tree_sink: &mut dyn TreeSink) {
//         let Reparser(r) = self;
//         let mut p = parser::Parser::new(token_source);
//         r(&mut p);
//         let events = p.finish();
//         event::process(tree_sink, events);
//     }
// }
