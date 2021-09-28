use crate::grammar::params::param_list;
use crate::grammar::types::type_;
use crate::grammar::{block_expr, name, name_r};
use crate::marker::Marker;
use crate::parser::Parser;
use crate::SyntaxKind::{self, *};
use crate::{TokenSet, T};

pub(crate) const ITEM_RECOVERY_SET: TokenSet = TokenSet::new(&[
    // T![struct],
    // T![const],
    // T![let],
    // T![use],
    // T![module],
    T![fun],
    // T![public],
    // T![script],
    T![;],
]);

pub(crate) fn script(p: &mut Parser) {
    let m = p.start();

    assert!(p.at(T![script]));
    p.bump(T![script]);

    if p.at(T!['{']) {
        script_item_list(p);
    } else if !p.eat(T![;]) {
        p.error("expected `;` or `{`");
    }
    m.complete(p, SCRIPT);
}

pub(crate) fn module(p: &mut Parser) {
    let m = p.start();

    assert!(p.at(T![module]));
    p.bump(T![module]);

    name(p);
    if p.at(T!['{']) {
        module_item_list(p);
    } else if !p.eat(T![;]) {
        p.error("expected `;` or `{`");
    }
    m.complete(p, MODULE);
}

pub(crate) fn module_item_list(p: &mut Parser) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    while !(p.at(T!['}']) || p.at(EOF)) {
        item(p, maybe_module_item);
    }
    p.expect(T!['}']);
    m.complete(p, MODULE_ITEM_LIST);
}

pub(crate) fn script_item_list(p: &mut Parser) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    while !(p.at(T!['}']) || p.at(EOF)) {
        item(p, maybe_script_item);
    }
    p.expect(T!['}']);
    m.complete(p, SCRIPT_ITEM_LIST);
}

// pub(crate) fn module_contents(p: &mut Parser) {
//     while !(p.at(T!['}']) || p.at(EOF)) {
//         item(p);
//     }
// }

pub(crate) fn item<F>(p: &mut Parser, maybe_item: F)
where
    F: Fn(&mut Parser, Marker) -> Result<(), Marker>,
{
    let m = p.start();
    let m = match maybe_item(p, m) {
        Ok(()) => {
            if p.at(T![;]) {
                p.err_and_bump(
                    "expected item, found `;`\n\
                     consider removing this semicolon",
                );
            }
            return;
        }
        Err(m) => m,
    };
    m.abandon(p);
    // if p.at(T!['{']) {
    //     error_block(p, "expected an item");
    // } else if p.at(T!['}']) && !stop_on_r_curly {
    //     let e = p.start();
    //     p.error("unmatched `}`");
    //     p.bump(T!['}']);
    //     e.complete(p, ERROR);
    if !p.at(EOF) && !p.at(T!['}']) {
        p.err_and_bump("expected an item");
    } else {
        p.error("expected an item");
    }
}

/// Try to parse an item, completing `m` in case of success.
pub(crate) fn maybe_module_item(p: &mut Parser, m: Marker) -> Result<(), Marker> {
    match p.current() {
        T![fun] => {
            fun(p);
            m.complete(p, FUN);
        }
        _ => {
            p.error("expected an item");
            m.complete(p, ERROR);
        }
    }
    Ok(())
}

/// Try to parse an item, completing `m` in case of success.
pub(crate) fn maybe_script_item(p: &mut Parser, m: Marker) -> Result<(), Marker> {
    match p.current() {
        T![fun] => {
            fun(p);
            m.complete(p, FUN);
        }
        _ => {
            p.error("expected an item");
            m.complete(p, ERROR);
        }
    }
    Ok(())
}

pub(crate) fn fun(p: &mut Parser) {
    assert!(p.at(T![fun]));
    p.bump(T![fun]);

    name_r(p, ITEM_RECOVERY_SET);

    if p.at(T!['(']) {
        param_list(p);
    } else {
        p.error("expected function arguments");
    }

    // test function_ret_type
    // fn foo() {}
    // fn bar() -> () {}
    opt_ret_type(p);

    block_expr(p);
}

fn opt_ret_type(p: &mut Parser) -> bool {
    if p.at(T![:]) {
        let m = p.start();
        p.bump(T![:]);
        type_(p);
        m.complete(p, RET_TYPE);
        true
    } else {
        false
    }
}
