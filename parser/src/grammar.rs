mod expressions;
mod items;

use crate::T;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::grammar::expressions::{stmt, StmtWithSemi};

pub(crate) fn root(p: &mut Parser) {
    let m = p.start();
    while !(p.at(T!['}']) || p.at(EOF)) {
        stmt(p, StmtWithSemi::Optional);
    }
    m.complete(p, SOURCE_FILE);
}