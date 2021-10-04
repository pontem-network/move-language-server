use crate::grammar::paths;
use crate::grammar::paths::is_path_start;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::TokenSet;

pub(super) const TYPE_FIRST: TokenSet = paths::PATH_FIRST.union(TokenSet::new(&[T![&]]));

const TYPE_RECOVERY_SET: TokenSet = TokenSet::new(&[
    T![')'],
    T![,],
    // L_DOLLAR,
]);

pub(super) fn ascription(p: &mut Parser) {
    assert!(p.at(T![:]));
    p.bump(T![:]);
    type_(p)
}

pub(crate) fn type_(p: &mut Parser) {
    let allow_bounds = false;
    match p.current() {
        T![&] => ref_type(p),
        _ if is_path_start(p) => path_type(p),
        _ => {
            p.err_recover("expected type", TYPE_RECOVERY_SET);
        }
    }
}

pub(crate) fn ref_type(p: &mut Parser) {
    assert!(p.at(T![&]));
    let m = p.start();
    p.bump(T![&]);
    p.eat(T![mut]);
    type_(p);
    m.complete(p, REF_TYPE);
}

pub(crate) fn path_type(p: &mut Parser) {
    assert!(is_path_start(p));
    let m = p.start();
    // let m = p.start();

    paths::type_path(p);

    // m.abandon(p);
    // let kind = PATH_TYPE;

    m.complete(p, PATH_TYPE);

    // if allow_bounds {
    //     opt_type_bounds_as_dyn_trait_type(p, path);
    // }
}
