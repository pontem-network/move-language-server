use crate::parser::Parser;
use crate::SyntaxKind;

pub(crate) fn root(p: &mut Parser) {
    let m = p.start();
    m.complete(p, SyntaxKind::SOURCE_FILE);
}
