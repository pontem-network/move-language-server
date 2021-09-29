mod expressions;
mod items;
mod params;
mod paths;
mod types;
mod generic_params;
mod patterns;

use crate::grammar::expressions::{stmt, StmtWithSemi};
use crate::grammar::items::{item, module, script};
use crate::marker::CompletedMarker;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::TokenSet;

pub(crate) fn block_expr(p: &mut Parser) {
    if !p.at(T!['{']) {
        p.error("expected a block");
        return;
    }
    block_expr_unchecked(p);
}

pub(crate) fn block_expr_unchecked(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    block_contents(p);
    p.expect(T!['}']);
    m.complete(p, BLOCK_EXPR)
}

pub(crate) fn block_contents(p: &mut Parser) {
    while !(p.at(T!['}']) || p.at(EOF)) {
        stmt(p, StmtWithSemi::Optional);
    }
}

fn name(p: &mut Parser) {
    name_r(p, TokenSet::EMPTY)
}

fn name_r(p: &mut Parser, recovery: TokenSet) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME);
    } else {
        p.err_recover("expected a name", recovery);
    }
}

fn name_ref(p: &mut Parser) {
    if p.at(IDENT) {
        let m = p.start();
        p.bump(IDENT);
        m.complete(p, NAME_REF);
    } else {
        p.err_and_bump("expected identifier");
    }
}

pub(crate) fn root(p: &mut Parser) {
    let m = p.start();
    while !(p.at(EOF)) {
        if p.at(T![module]) {
            module(p);
        } else if p.at(T![script]) {
            script(p);
        } else if p.current().is_trivia() {
            p.bump_any()
        } else {
            p.error("expected module or script");
        }
    }
    m.complete(p, SOURCE_FILE);
}

fn error_block(p: &mut Parser, message: &str) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.error(message);
    p.bump(T!['{']);
    expressions::expr_block_contents(p);
    p.eat(T!['}']);
    m.complete(p, ERROR);
}
