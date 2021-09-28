mod atom;

use crate::grammar::expressions::atom::{atom_expr, literal};
use crate::grammar::{name, types};
use crate::marker::{CompletedMarker, Marker};
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::{TokenSet, T};

pub(crate) enum StmtWithSemi {
    Yes,
    No,
    Optional,
}

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

// test let_stmt
// fn foo() {
//     let a;
//     let b: i32;
//     let c = 92;
//     let d: i32 = 92;
//     let e: !;
//     let _: ! = {};
//     let f = #[attr]||{};
// }
fn let_stmt(p: &mut Parser, m: Marker, with_semi: StmtWithSemi) {
    assert!(p.at(T![let]));
    p.bump(T![let]);
    name(p);

    if p.at(T![:]) {
        types::ascription(p);
    }
    if p.eat(T![=]) {
        expr(p);
    }

    match with_semi {
        StmtWithSemi::Yes => {
            p.expect(T![;]);
        }
        StmtWithSemi::No => {}
        StmtWithSemi::Optional => {
            if p.at(T![;]) {
                p.eat(T![;]);
            }
        }
    }
    m.complete(p, LET_STMT);
}

pub(crate) fn stmt(p: &mut Parser, with_semi: StmtWithSemi) {
    let m = p.start();
    if p.at(T![let]) {
        let_stmt(p, m, with_semi);
        return;
    }
    expr(p);
    match with_semi {
        StmtWithSemi::Yes => {
            p.expect(T![;]);
        }
        StmtWithSemi::No => {}
        StmtWithSemi::Optional => {
            if p.at(T![;]) {
                p.eat(T![;]);
            }
        }
    };
    m.complete(p, EXPR_STMT);
    // p.eat(T![;]);
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
