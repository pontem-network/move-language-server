//! This module generates AST datatype used by rust-analyzer.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of newtype
//! wrappers around `SyntaxNode` which implement `syntax::AstNode`.

pub(crate) mod gen_nodes;
pub(crate) mod gen_tokens;

use std::{
    collections::{BTreeSet, HashSet},
    fmt::Write,
};

use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule};

use stdx::{to_lower_snake_case, to_pascal_case, to_upper_snake_case};

use crate::syntax_kind::{SymbolKindsSrc, KINDS_SRC};
use crate::utils::{pluralize, reformat};

#[derive(Default, Debug)]
pub(crate) struct GrammarAST {
    pub(crate) tokens: Vec<String>,
    pub(crate) nodes: Vec<GrammarNode>,
    pub(crate) enums: Vec<GrammarEnum>,
}

#[derive(Debug)]
pub(crate) struct GrammarNode {
    pub(crate) doc: Vec<String>,
    pub(crate) name: String,
    pub(crate) traits: Vec<String>,
    pub(crate) fields: Vec<GrammarNodeField>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum GrammarNodeField {
    Token(String),
    Node { name: String, ty: String, cardinality: Cardinality },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Cardinality {
    Optional,
    Many,
}

#[derive(Debug)]
pub(crate) struct GrammarEnum {
    pub(crate) doc: Vec<String>,
    pub(crate) name: String,
    pub(crate) traits: Vec<String>,
    pub(crate) variants: Vec<String>,
}

impl GrammarNodeField {
    fn is_many(&self) -> bool {
        matches!(self, GrammarNodeField::Node { cardinality: Cardinality::Many, .. })
    }
    fn token_kind(&self) -> Option<proc_macro2::TokenStream> {
        match self {
            GrammarNodeField::Token(token) => {
                let token: proc_macro2::TokenStream = token.parse().unwrap();
                Some(quote! { T![#token] })
            }
            _ => None,
        }
    }
    fn method_name(&self) -> proc_macro2::Ident {
        match self {
            GrammarNodeField::Token(name) => {
                for (token_symbol, token_name) in KINDS_SRC.punct {
                    let grammar_token_name = name.as_str().trim_matches('\'');
                    if grammar_token_name == *token_symbol {
                        return format_ident!("{}_token", token_name.to_ascii_lowercase());
                    }
                }
                format_ident!("{}_token", name)
            }
            GrammarNodeField::Node { name, .. } => {
                if name == "type" {
                    format_ident!("ty")
                } else {
                    format_ident!("{}", name)
                }
            }
        }
    }
    fn ty(&self) -> proc_macro2::Ident {
        match self {
            GrammarNodeField::Token(_) => format_ident!("SyntaxToken"),
            GrammarNodeField::Node { ty, .. } => format_ident!("{}", ty),
        }
    }
}

pub(crate) fn lower(grammar: &Grammar) -> GrammarAST {
    let mut grammar_ast = GrammarAST::default();

    grammar_ast.tokens = "Whitespace Comment HexString ByteString IntegerNumber"
        .split_ascii_whitespace()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let nodes = grammar.iter().collect::<Vec<_>>();

    for &node in &nodes {
        let name = grammar[node].name.clone();
        let rule = &grammar[node].rule;
        match lower_enum(grammar, rule) {
            Some(variants) => {
                let enum_src = GrammarEnum { doc: Vec::new(), name, traits: Vec::new(), variants };
                grammar_ast.enums.push(enum_src);
            }
            None => {
                let mut fields = Vec::new();
                lower_rule(&mut fields, grammar, None, rule);
                grammar_ast.nodes.push(GrammarNode {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    fields,
                });
            }
        }
    }

    deduplicate_fields(&mut grammar_ast);
    extract_enums(&mut grammar_ast);
    extract_struct_traits(&mut grammar_ast);
    extract_enum_traits(&mut grammar_ast);
    grammar_ast
}

fn lower_enum(grammar: &Grammar, rule: &Rule) -> Option<Vec<String>> {
    let alternatives = match rule {
        Rule::Alt(it) => it,
        _ => return None,
    };
    let mut variants = Vec::new();
    for alternative in alternatives {
        match alternative {
            Rule::Node(it) => variants.push(grammar[*it].name.clone()),
            Rule::Token(it) if grammar[*it].name == ";" => (),
            _ => return None,
        }
    }
    Some(variants)
}

fn lower_rule(
    acc: &mut Vec<GrammarNodeField>,
    grammar: &Grammar,
    label: Option<&String>,
    rule: &Rule,
) {
    if lower_comma_list(acc, grammar, label, rule) {
        return;
    }

    match rule {
        Rule::Node(node) => {
            let ty = grammar[*node].name.clone();
            let name = label.cloned().unwrap_or_else(|| to_lower_snake_case(&ty));
            let field = GrammarNodeField::Node { name, ty, cardinality: Cardinality::Optional };
            acc.push(field);
        }
        Rule::Token(token) => {
            assert!(label.is_none());
            let mut name = grammar[*token].name.clone();
            if name != "integer_number" && name != "hex_string" && name != "byte_string" {
                if "[]{}()".contains(&name) {
                    name = format!("'{}'", name);
                }
                let field = GrammarNodeField::Token(name);
                acc.push(field);
            }
        }
        Rule::Rep(inner) => {
            if let Rule::Node(node) = &**inner {
                let ty = grammar[*node].name.clone();
                let name = label.cloned().unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
                let field = GrammarNodeField::Node { name, ty, cardinality: Cardinality::Many };
                acc.push(field);
                return;
            }
            panic!("unhandled rule: {:?}", rule)
        }
        Rule::Labeled { label: l, rule } => {
            assert!(label.is_none());
            // let manually_implemented = matches!(
            //     l.as_str(),
            //         "then_branch"
            //         | "else_branch"
            //         | "start"
            //         | "end"
            //         | "index"
            //         | "base"
            //         | "value"
            //         | "trait"
            //         | "self_ty"
            // );
            // if manually_implemented {
            //     return;
            // }
            lower_rule(acc, grammar, Some(l), rule);
        }
        Rule::Seq(rules) | Rule::Alt(rules) => {
            for rule in rules {
                lower_rule(acc, grammar, label, rule)
            }
        }
        Rule::Opt(rule) => lower_rule(acc, grammar, label, rule),
    }
}

// (T (',' T)* ','?)
fn lower_comma_list(
    acc: &mut Vec<GrammarNodeField>,
    grammar: &Grammar,
    label: Option<&String>,
    rule: &Rule,
) -> bool {
    let rule = match rule {
        Rule::Seq(it) => it,
        _ => return false,
    };
    let (node, repeat, trailing_comma) = match rule.as_slice() {
        [Rule::Node(node), Rule::Rep(repeat), Rule::Opt(trailing_comma)] => {
            (node, repeat, trailing_comma)
        }
        _ => return false,
    };
    let repeat = match &**repeat {
        Rule::Seq(it) => it,
        _ => return false,
    };
    match repeat.as_slice() {
        [comma, Rule::Node(n)] if comma == &**trailing_comma && n == node => (),
        _ => return false,
    }
    let ty = grammar[*node].name.clone();
    let name = label.cloned().unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
    let field = GrammarNodeField::Node { name, ty, cardinality: Cardinality::Many };
    acc.push(field);
    true
}

fn deduplicate_fields(ast: &mut GrammarAST) {
    for node in &mut ast.nodes {
        let mut i = 0;
        'outer: while i < node.fields.len() {
            for j in 0..i {
                let f1 = &node.fields[i];
                let f2 = &node.fields[j];
                if f1 == f2 {
                    node.fields.remove(i);
                    continue 'outer;
                }
            }
            i += 1;
        }
    }
}

fn extract_enums(ast: &mut GrammarAST) {
    for node in &mut ast.nodes {
        for enm in &ast.enums {
            let mut to_remove = Vec::new();
            for (i, field) in node.fields.iter().enumerate() {
                let ty = field.ty().to_string();
                if enm.variants.iter().any(|it| it == &ty) {
                    to_remove.push(i);
                }
            }
            if to_remove.len() == enm.variants.len() {
                node.remove_field(to_remove);
                let ty = enm.name.clone();
                let name = to_lower_snake_case(&ty);
                node.fields.push(GrammarNodeField::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Optional,
                });
            }
        }
    }
}

fn extract_struct_traits(ast: &mut GrammarAST) {
    let traits: &[(&str, &[&str])] = &[
        ("AttrsOwner", &["attrs"]),
        ("NameOwner", &["name"]),
        ("VisibilityOwner", &["visibility"]),
        ("GenericParamsOwner", &["generic_param_list", "where_clause"]),
        ("TypeBoundsOwner", &["type_bound_list", "colon_token"]),
        ("ModuleItemOwner", &["items"]),
        ("LoopBodyOwner", &["label", "loop_body"]),
        ("ArgListOwner", &["arg_list"]),
    ];

    for node in &mut ast.nodes {
        for (name, methods) in traits {
            extract_struct_trait(node, name, methods);
        }
    }
}

fn extract_struct_trait(node: &mut GrammarNode, trait_name: &str, methods: &[&str]) {
    let mut to_remove = Vec::new();
    for (i, field) in node.fields.iter().enumerate() {
        let method_name = field.method_name().to_string();
        if methods.iter().any(|&it| it == method_name) {
            to_remove.push(i);
        }
    }
    if to_remove.len() == methods.len() {
        node.traits.push(trait_name.to_string());
        node.remove_field(to_remove);
    }
}

fn extract_enum_traits(ast: &mut GrammarAST) {
    for enm in &mut ast.enums {
        if enm.name == "Stmt" {
            continue;
        }
        let nodes = &ast.nodes;
        let mut variant_traits = enm
            .variants
            .iter()
            .map(|var| nodes.iter().find(|it| &it.name == var).unwrap())
            .map(|node| node.traits.iter().cloned().collect::<BTreeSet<_>>());

        let mut enum_traits = match variant_traits.next() {
            Some(it) => it,
            None => continue,
        };
        for traits in variant_traits {
            enum_traits = enum_traits.intersection(&traits).cloned().collect();
        }
        enm.traits = enum_traits.into_iter().collect();
    }
}

impl GrammarNode {
    fn remove_field(&mut self, to_remove: Vec<usize>) {
        to_remove.into_iter().rev().for_each(|idx| {
            self.fields.remove(idx);
        });
    }
}
