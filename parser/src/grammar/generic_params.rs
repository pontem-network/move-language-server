use super::*;
use crate::marker::Marker;

pub(super) fn opt_generic_param_list(p: &mut Parser) {
    if p.at(T![<]) {
        generic_param_list(p);
    }
}

// test generic_param_list
// fn f<T: Clone>() {}
fn generic_param_list(p: &mut Parser) {
    assert!(p.at(T![<]));
    let m = p.start();
    p.bump(T![<]);

    while !p.at(EOF) && !p.at(T![>]) {
        generic_param(p);
        if !p.at(T![>]) && !p.expect(T![,]) {
            break;
        }
    }
    p.expect(T![>]);
    m.complete(p, GENERIC_PARAM_LIST);
}

fn generic_param(p: &mut Parser) {
    let m = p.start();
    match p.current() {
        IDENT => type_param(p, m),
        _ => {
            m.abandon(p);
            p.err_and_bump("expected type parameter")
        }
    }
}

// test type_param
// fn f<T: Clone>() {}
fn type_param(p: &mut Parser, m: Marker) {
    assert!(p.at(IDENT));
    name(p);
    if p.at(T![:]) {
        bounds(p);
    }
    // if p.at(T![=]) {
    //     // test type_param_default
    //     // struct S<T = i32>;
    //     p.bump(T![=]);
    //     types::type_(p)
    // }
    m.complete(p, TYPE_PARAM);
}

// test type_param_bounds
// struct S<T: 'a + ?Sized + (Copy)>;
pub(super) fn bounds(p: &mut Parser) {
    assert!(p.at(T![:]));
    p.bump(T![:]);
    bounds_without_colon(p);
}

pub(super) fn bounds_without_colon(p: &mut Parser) {
    let m = p.start();
    bounds_without_colon_m(p, m);
}

pub(super) fn bounds_without_colon_m(p: &mut Parser, marker: Marker) -> CompletedMarker {
    while type_bound(p) {
        if !p.eat(T![+]) {
            break;
        }
    }
    marker.complete(p, TYPE_BOUND_LIST)
}

fn type_bound(p: &mut Parser) -> bool {
    let m = p.start();
    // let has_paren = p.eat(T!['(']);
    // p.eat(T![?]);
    match p.current() {
        // T![for] => types::for_type(p, false),
        IDENT => name(p),
        _ => {
            m.abandon(p);
            return false;
        }
    }
    // if has_paren {
    //     p.expect(T![')']);
    // }
    m.complete(p, TYPE_BOUND);

    true
}
