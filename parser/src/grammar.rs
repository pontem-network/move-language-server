mod expressions;

use crate::parser::Parser;
use crate::SyntaxKind::{self, *};

pub(crate) fn root(p: &mut Parser) {
    let m = p.start();
    expressions::expr(p);
    m.complete(p, SOURCE_FILE);
}

// fn name_ref_or_index(p: &mut Parser) {
//     assert!(p.at(IDENT) || p.at(INTEGER_NUMBER));
//     let m = p.start();
//     p.bump_any();
//     m.complete(p, NAME_REF);
// }
