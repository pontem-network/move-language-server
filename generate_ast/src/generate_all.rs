use crate::ast::{generate_nodes, generate_tokens, lower};
use crate::syntax_kind::{generate_syntax_kinds, KINDS_SRC};
use test_utils::{ensure_file_contents, project_root};
use ungrammar::Grammar;

fn move_grammar() -> Grammar {
    let move_ungram_src = include_str!("./move.ungram");
    move_ungram_src.parse().unwrap()
}

#[test]
fn generate_ast() {
    let grammar = move_grammar();
    let grammar_ast = lower(&grammar);

    let syntax_kinds_file = project_root().join("parser/src/syntax_kind/generated.rs");
    let syntax_kinds = generate_syntax_kinds(KINDS_SRC);
    ensure_file_contents(syntax_kinds_file.as_path(), &syntax_kinds);

    let ast_tokens_file = project_root().join("syntax/src/ast/tokens.rs");
    let contents = generate_tokens(&grammar_ast);
    ensure_file_contents(ast_tokens_file.as_path(), &contents);

    let ast_nodes_file = project_root().join("syntax/src/ast/nodes.rs");
    let contents = generate_nodes(KINDS_SRC, &grammar_ast);
    ensure_file_contents(ast_nodes_file.as_path(), &contents);
}
