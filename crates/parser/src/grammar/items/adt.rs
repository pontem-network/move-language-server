use super::*;
use crate::grammar::{generic_params, types};

pub(crate) fn struct_(p: &mut Parser, m: Marker) {
    p.bump(T![struct]);
    name_r(p, ITEM_RECOVERY_SET);
    generic_params::opt_generic_param_list(p);
    match p.current() {
        // T![where] => {
        //     generic_params::opt_where_clause(p);
        //     match p.current() {
        //         T![;] => p.bump(T![;]),
        //         T!['{'] => record_field_list(p),
        //         _ => {
        //             //FIXME: special case `(` error message
        //             p.error("expected `;` or `{`");
        //         }
        //     }
        // }
        T!['{'] => record_field_list(p),
        // test unit_struct
        // struct S;
        T![;] => {
            p.bump(T![;]);
        }
        // test tuple_struct
        // struct S(String, usize);
        // T!['('] if is_struct => {
        //     tuple_field_list(p);
        //     // test tuple_struct_where
        //     // struct S<T>(T) where T: Clone;
        //     generic_params::opt_where_clause(p);
        //     p.expect(T![;]);
        // }
        _ => p.error("expected `;` or `{`"),
    }
    m.complete(p, STRUCT);
}

// test record_field_list
// struct S { a: i32, b: f32 }
pub(crate) fn record_field_list(p: &mut Parser) {
    assert!(p.at(T!['{']));
    let m = p.start();
    p.bump(T!['{']);
    while !p.at(T!['}']) && !p.at(EOF) {
        // if p.at(T!['{']) {
        //     error_block(p, "expected field");
        //     continue;
        // }
        record_field(p);
        if !p.at(T!['}']) {
            p.expect(T![,]);
        }
    }
    p.expect(T!['}']);
    m.complete(p, RECORD_FIELD_LIST);

    fn record_field(p: &mut Parser) {
        let m = p.start();
        if p.at(IDENT) {
            name(p);
            p.expect(T![:]);
            types::type_(p);
            m.complete(p, RECORD_FIELD);
        } else {
            m.abandon(p);
            p.err_and_bump("expected field declaration");
        }
    }
}
