mod atom;

use crate::grammar::expressions::atom::{atom_expr, literal};
use crate::marker::CompletedMarker;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::{TokenSet, T};

pub(super) fn expr(p: &mut Parser) -> Option<CompletedMarker> {
    expr_bp(p, 1)
}

// Parses expression with binding power of at least bp.
fn expr_bp(p: &mut Parser, bp: u8) -> Option<CompletedMarker> {
    let mut lhs = lhs(p)?;
    loop {
        let (op_bp, op) = current_op(p);
        if op_bp < bp {
            break;
        }
        let m = lhs.precede(p);
        p.bump(op);
        expr_bp(p, op_bp + 1);
        // if expr_bp(p, op_bp + 1).is_none() {
        //     p.error_and_skip_until("expected expression", TokenSet::new(&[T![')']]));
        //     m.complete(p, BIN_EXPR);
        //     return None;
        // }
        lhs = m.complete(p, BIN_EXPR);
    }
    Some(lhs)
}

fn lhs(p: &mut Parser) -> Option<CompletedMarker> {
    let m;
    let kind = match p.current() {
        T![*] | T![!] | T![-] => {
            m = p.start();
            p.bump_any();
            PREFIX_EXPR
        }
        _ => {
            let lhs = atom_expr(p)?;
            return Some(lhs);
        }
    };
    // parse the interior of the unary expression
    expr_bp(p, 255);
    Some(m.complete(p, kind))
}

/// Binding powers of operators for a Pratt parser.
///
/// See <https://www.oilshell.org/blog/2016/11/03.html>
#[rustfmt::skip]
fn current_op(p: &Parser) -> (u8, SyntaxKind) {
    const NOT_AN_OP: (u8, SyntaxKind) = (0, T![@]);
    match p.current() {
        // T![|] if p.at(T![||])  => (3,  T![||]),
        // T![|] if p.at(T![|=])  => (1,  T![|=]),
        T![|]                  => (6,  T![|]),
        // T![>] if p.at(T![>>=]) => (1,  T![>>=]),
        // T![>] if p.at(T![>>])  => (9,  T![>>]),
        // T![>] if p.at(T![>=])  => (5,  T![>=]),
        T![>]                  => (5,  T![>]),
        // T![=] if p.at(T![=>])  => NOT_AN_OP,
        // T![=] if p.at(T![==])  => (5,  T![==]),
        T![=]                  => (1,  T![=]),
        // T![<] if p.at(T![<=])  => (5,  T![<=]),
        // T![<] if p.at(T![<<=]) => (1,  T![<<=]),
        // T![<] if p.at(T![<<])  => (9,  T![<<]),
        T![<]                  => (5,  T![<]),
        // T![+] if p.at(T![+=])  => (1,  T![+=]),
        T![+]                  => (10, T![+]),
        // T![^] if p.at(T![^=])  => (1,  T![^=]),
        T![^]                  => (7,  T![^]),
        // T![%] if p.at(T![%=])  => (1,  T![%=]),
        T![%]                  => (11, T![%]),
        // T![&] if p.at(T![&=])  => (1,  T![&=]),
        // T![&] if p.at(T![&&])  => (4,  T![&&]),
        T![&]                  => (8,  T![&]),
        // T![/] if p.at(T![/=])  => (1,  T![/=]),
        T![/]                  => (11, T![/]),
        // T![*] if p.at(T![*=])  => (1,  T![*=]),
        T![*]                  => (11, T![*]),
        // T![.] if p.at(T![..=]) => (2,  T![..=]),
        // T![.] if p.at(T![..])  => (2,  T![..]),
        // T![!] if p.at(T![!=])  => (5,  T![!=]),
        // T![-] if p.at(T![-=])  => (1,  T![-=]),
        T![-]                  => (10, T![-]),
        // T![as]                 => (12, T![as]),

        _                      => NOT_AN_OP
    }
}
