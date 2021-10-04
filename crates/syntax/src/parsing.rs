use crate::parsing::text_tree_sink::TextTreeSink;
use crate::syntax_error::SyntaxError;
use parser::{event, Lexer};
use rowan::GreenNode;

mod text_tree_sink;

pub(crate) fn parse_text(text: &str) -> (GreenNode, Vec<SyntaxError>) {
    let mut lexer = Lexer::new(text);
    let (events, tokens) = parser::parse_with_lexer(&mut lexer);

    let mut tree_sink = TextTreeSink::new(text, &tokens);
    event::process(&mut tree_sink, events);

    let (tree, mut parser_errors) = tree_sink.finish();
    // parser::parse(&mut lexer, &mut tree_sink);
    // parser_errors.extend(lexer_errors);

    (tree, parser_errors)
}
