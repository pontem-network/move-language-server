#![allow(unused)]

use super::*;

// test expr_literals
// fn foo() {
//     let _ = true;
//     let _ = false;
//     let _ = 1;
//     let _ = 2.0;
//     let _ = b'a';
//     let _ = 'b';
//     let _ = "c";
//     let _ = r"d";
//     let _ = b"e";
//     let _ = br"f";
// }
pub(crate) const LITERAL_FIRST: TokenSet = TokenSet::new(&[
    T![true],
    T![false],
    INTEGER_NUMBER,
    // BYTE,
    // CHAR,
    HEX_STRING,
    BYTE_STRING,
]);

pub(crate) fn literal(p: &mut Parser) -> Option<CompletedMarker> {
    if !p.at_ts(LITERAL_FIRST) {
        return None;
    }
    let m = p.start();
    p.bump_any();
    Some(m.complete(p, LITERAL))
}

// E.g. for after the break in `if break {}`, this should not match
pub(super) const ATOM_EXPR_FIRST: TokenSet = LITERAL_FIRST.union(TokenSet::new(&[
    T!['('],
    T!['{'],
    T!['['],
    // L_DOLLAR,
    T![|],
    T![move],
    // T![box],
    // T![if],
    // T![while],
    // T![match],
    // T![unsafe],
    T![return],
    // T![yield],
    // T![break],
    // T![continue],
    // T![async],
    // T![try],
    // T![const],
    // T![loop],
    // T![for],
    // LIFETIME_IDENT,
]));

const EXPR_RECOVERY_SET: TokenSet = TokenSet::new(&[LET_KW]);

pub(super) fn atom_expr(p: &mut Parser) -> Option<CompletedMarker> {
    if let Some(m) = literal(p) {
        return Some(m);
    }
    if p.current() == IDENT {
        return Some(path_expr(p));
    }
    let done = match p.current() {
        T!['('] => paren_expr(p),
        _ => {
            let done =
                p.error_and_skip_until("expected expression", TokenSet::new(&[T![;], T![')']]));
            return Some(done);
        }
    };
    Some(done)
}

fn path_expr(p: &mut Parser) -> CompletedMarker {
    let m = p.start();
    p.bump(IDENT);
    m.complete(p, PATH_EXPR)
}

// test tuple_expr
// fn foo() {
//     ();
//     (1);
//     (1,);
// }
fn paren_expr(p: &mut Parser) -> CompletedMarker {
    assert!(p.at(T!['(']));
    let m = p.start();
    p.expect(T!['(']);

    while !p.at(EOF) && !p.at(T![')']) {
        // saw_expr = true;

        // test tuple_attrs
        // const A: (i64, i64) = (1, #[cfg(test)] 2);
        let expr = expr(p);
        if expr.is_none() {
            p.error_and_skip_until("expected expression", TokenSet::new(&[T![')']]));
            return m.complete(p, PAREN_EXPR);
        }

        // if !expr_with_attrs(p) {
        //     break;
        // }

        // if !p.at(T![')'])
        // if !p.at(T![')']) {
        //     saw_comma = true;
        //     p.expect(T![,]);
        // }
    }
    // assert!(!saw_comma);

    p.expect(T![')']);
    m.complete(p, PAREN_EXPR)
}