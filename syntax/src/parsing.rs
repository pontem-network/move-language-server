use rowan::GreenNode;
use crate::syntax_error::SyntaxError;
use crate::parsing::text_token_source::Lexer;
use crate::parsing::text_tree_sink::TextTreeSink;

mod text_token_source;
mod text_tree_sink;

pub(crate) fn parse_text(text: &str) -> (GreenNode, Vec<SyntaxError>) {
    let mut lexer = Lexer::new(text);
    let mut tree_sink = TextTreeSink::new(text, &tokens);

    parser::parse(&mut lexer, &mut tree_sink);

    let (tree, mut parser_errors) = tree_sink.finish();
    parser_errors.extend(lexer_errors);

    (tree, parser_errors)
}