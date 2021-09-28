use crate::grammar::{name, name_r, types};
use crate::marker::Marker;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::{TokenSet, T};

const PARAM_FIRST: TokenSet = types::TYPE_FIRST;

pub(crate) fn param_list(p: &mut Parser) {
    let list_marker = p.start();
    p.bump(T!['(']);

    let mut param_marker = None;
    while !p.at(EOF) && !p.at(T![')']) {
        // test param_outer_arg
        // fn f(#[attr1] pat: Type) {}
        let m = match param_marker.take() {
            Some(m) => m,
            None => {
                let m = p.start();
                // attributes::outer_attrs(p);
                m
            }
        };

        if !p.at_ts(PARAM_FIRST) {
            p.error("expected value parameter");
            m.abandon(p);
            break;
        }
        let param = param(p, m);
        if !p.at(T![')']) {
            p.expect(T![,]);
        }
    }

    if let Some(m) = param_marker {
        m.abandon(p);
    }

    p.expect(T![')']);
    list_marker.complete(p, PARAM_LIST);
}

fn param(p: &mut Parser, m: Marker) {
    name(p);
    if p.at(T![:]) {
        types::ascription(p)
    } else {
        // test_err missing_fn_param_type
        // fn f(x y: i32, z, t: i32) {}
        p.error("missing type for function parameter")
    }
    m.complete(p, PARAM);
}
