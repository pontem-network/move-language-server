//! This module generates AST datatype used by rust-analyzer.
//!
//! Specifically, it generates the `SyntaxKind` enum and a number of newtype
//! wrappers around `SyntaxNode` which implement `syntax::AstNode`.

use std::path::{Path, PathBuf};
use std::{
    collections::{BTreeSet, HashSet},
    fmt::Write,
    fs,
};

use proc_macro2::{Punct, Spacing};
use quote::{format_ident, quote};
use ungrammar::{Grammar, Rule};
use xshell::cmd;
use xshell::pushenv;

use crate::syntax_kind::{generate_syntax_kinds, SymbolKindsSrc, KINDS_SRC};
use crate::utils::{
    cargo_project_root, ensure_file_contents, pluralize, reformat, to_lower_snake_case,
    to_pascal_case, to_upper_snake_case,
};

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
    Node {
        name: String,
        ty: String,
        cardinality: Cardinality,
    },
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

pub(crate) fn generate_tokens(grammar: &GrammarAST) -> String {
    let tokens = grammar.tokens.iter().map(|token| {
        let name = format_ident!("{}", token);
        let kind = format_ident!("{}", to_upper_snake_case(token));
        quote! {
            #[derive(Debug, Clone, PartialEq, Eq, Hash)]
            pub struct #name {
                pub(crate) syntax: SyntaxToken,
            }
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    std::fmt::Display::fmt(&self.syntax, f)
                }
            }
            impl AstToken for #name {
                fn can_cast(kind: SyntaxKind) -> bool { kind == #kind }
                fn cast(syntax: SyntaxToken) -> Option<Self> {
                    if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                }
                fn syntax(&self) -> &SyntaxToken { &self.syntax }
            }
        }
    });
    reformat(
        quote! {
            use crate::{SyntaxKind::{self, *}, syntax_node::SyntaxToken, ast::AstToken};
            #(#tokens)*
        }
        .to_string(),
    )
    .replace("#[derive", "\n#[derive")
}

pub(crate) fn generate_nodes(kinds: SymbolKindsSrc<'_>, grammar: &GrammarAST) -> String {
    let (node_defs, node_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .nodes
        .iter()
        .map(|node| {
            let name = format_ident!("{}", node.name);
            let kind = format_ident!("{}", to_upper_snake_case(&node.name));
            let traits = node.traits.iter().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl ast::#trait_name for #name {})
            });

            let methods = node.fields.iter().map(|field| {
                let method_name = field.method_name();
                let ty = field.ty();

                if field.is_many() {
                    quote! {
                        pub fn #method_name(&self) -> AstChildren<#ty> {
                            support::children(&self.syntax)
                        }
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::token(&self.syntax, #token_kind)
                        }
                    }
                } else {
                    quote! {
                        pub fn #method_name(&self) -> Option<#ty> {
                            support::child(&self.syntax)
                        }
                    }
                }
            });
            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub struct #name {
                        pub(crate) syntax: SyntaxNode,
                    }

                    #(#traits)*

                    impl #name {
                        #(#methods)*
                    }
                },
                quote! {
                    impl AstNode for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            kind == #kind
                        }
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                        }
                        fn syntax(&self) -> &SyntaxNode { &self.syntax }
                    }
                },
            )
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .enums
        .iter()
        .map(|en| {
            let variants: Vec<_> = en
                .variants
                .iter()
                .map(|var| format_ident!("{}", var))
                .collect();
            let name = format_ident!("{}", en.name);
            let kinds: Vec<_> = variants
                .iter()
                .map(|name| format_ident!("{}", to_upper_snake_case(&name.to_string())))
                .collect();
            let traits = en.traits.iter().map(|trait_name| {
                let trait_name = format_ident!("{}", trait_name);
                quote!(impl ast::#trait_name for #name {})
            });

            let ast_node = if en.name == "Stmt" {
                quote! {}
            } else {
                quote! {
                    impl AstNode for #name {
                        fn can_cast(kind: SyntaxKind) -> bool {
                            match kind {
                                #(#kinds)|* => true,
                                _ => false,
                            }
                        }
                        fn cast(syntax: SyntaxNode) -> Option<Self> {
                            let res = match syntax.kind() {
                                #(
                                #kinds => #name::#variants(#variants { syntax }),
                                )*
                                _ => return None,
                            };
                            Some(res)
                        }
                        fn syntax(&self) -> &SyntaxNode {
                            match self {
                                #(
                                #name::#variants(it) => &it.syntax,
                                )*
                            }
                        }
                    }
                }
            };

            (
                quote! {
                    #[pretty_doc_comment_placeholder_workaround]
                    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                    pub enum #name {
                        #(#variants(#variants),)*
                    }

                    #(#traits)*
                },
                quote! {
                    #(
                        impl From<#variants> for #name {
                            fn from(node: #variants) -> #name {
                                #name::#variants(node)
                            }
                        }
                    )*
                    #ast_node
                },
            )
        })
        .unzip();

    let enum_names = grammar.enums.iter().map(|it| &it.name);
    let node_names = grammar.nodes.iter().map(|it| &it.name);

    let display_impls = enum_names
        .chain(node_names.clone())
        .map(|it| format_ident!("{}", it))
        .map(|name| {
            quote! {
                impl std::fmt::Display for #name {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        std::fmt::Display::fmt(self.syntax(), f)
                    }
                }
            }
        });

    let defined_nodes: HashSet<_> = node_names.collect();

    for node in kinds
        .nodes
        .iter()
        .map(|kind| to_pascal_case(kind))
        .filter(|name| !defined_nodes.iter().any(|&it| it == name))
    {
        drop(node)
        // FIXME: restore this
        // eprintln!("Warning: node {} not defined in ast source", node);
    }

    let ast = quote! {
        use crate::syntax_node::{SyntaxNode, SyntaxToken};
        use crate::SyntaxKind::{self, *};
        use crate::T;
        use crate::ast::{self, AstNode, AstChildren, support};

        #(#node_defs)*
        #(#enum_defs)*
        #(#node_boilerplate_impls)*
        #(#enum_boilerplate_impls)*
        #(#display_impls)*
    };

    let ast = ast.to_string().replace("T ! [", "T![");

    let mut res = String::with_capacity(ast.len() * 2);

    let mut docs = grammar
        .nodes
        .iter()
        .map(|it| &it.doc)
        .chain(grammar.enums.iter().map(|it| &it.doc));

    for chunk in ast.split("# [pretty_doc_comment_placeholder_workaround] ") {
        res.push_str(chunk);
        if let Some(doc) = docs.next() {
            write_doc_comment(doc, &mut res);
        }
    }

    reformat(res)
    // sourcegen::add_preamble("sourcegen_ast", sourcegen::reformat(res))
}

fn write_doc_comment(contents: &[String], dest: &mut String) {
    for line in contents {
        writeln!(dest, "///{}", line).unwrap();
    }
}

impl GrammarNodeField {
    fn is_many(&self) -> bool {
        matches!(
            self,
            GrammarNodeField::Node {
                cardinality: Cardinality::Many,
                ..
            }
        )
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
                    if name.as_str() == *token_symbol {
                        return format_ident!("{}_token", token_name);
                    }
                }
                format_ident!("{}_token", name)
                //
                // let name = match name.as_str() {
                //     ";" => "semicolon",
                //     "->" => "thin_arrow",
                //     "'{'" => "l_curly",
                //     "'}'" => "r_curly",
                //     "'('" => "l_paren",
                //     "')'" => "r_paren",
                //     "'['" => "l_brack",
                //     "']'" => "r_brack",
                //     "<" => "l_angle",
                //     ">" => "r_angle",
                //     "=" => "eq",
                //     "!" => "excl",
                //     "*" => "star",
                //     "&" => "amp",
                //     "_" => "underscore",
                //     "." => "dot",
                //     ".." => "dotdot",
                //     "..." => "dotdotdot",
                //     "..=" => "dotdoteq",
                //     "=>" => "fat_arrow",
                //     "@" => "at",
                //     ":" => "colon",
                //     "::" => "coloncolon",
                //     "#" => "pound",
                //     "?" => "question_mark",
                //     "," => "comma",
                //     "|" => "pipe",
                //     _ => name,
                // };
                // format_ident!("{}_token", name)
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
    let mut res = GrammarAST::default();

    res.tokens = "Whitespace Comment HexString ByteString IntegerNumber"
        .split_ascii_whitespace()
        .map(|it| it.to_string())
        .collect::<Vec<_>>();

    let nodes = grammar.iter().collect::<Vec<_>>();

    for &node in &nodes {
        let name = grammar[node].name.clone();
        let rule = &grammar[node].rule;
        match lower_enum(grammar, rule) {
            Some(variants) => {
                let enum_src = GrammarEnum {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    variants,
                };
                res.enums.push(enum_src);
            }
            None => {
                let mut fields = Vec::new();
                lower_rule(&mut fields, grammar, None, rule);
                res.nodes.push(GrammarNode {
                    doc: Vec::new(),
                    name,
                    traits: Vec::new(),
                    fields,
                });
            }
        }
    }

    deduplicate_fields(&mut res);
    extract_enums(&mut res);
    extract_struct_traits(&mut res);
    extract_enum_traits(&mut res);
    res
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
            let field = GrammarNodeField::Node {
                name,
                ty,
                cardinality: Cardinality::Optional,
            };
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
                let name = label
                    .cloned()
                    .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
                let field = GrammarNodeField::Node {
                    name,
                    ty,
                    cardinality: Cardinality::Many,
                };
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
    let name = label
        .cloned()
        .unwrap_or_else(|| pluralize(&to_lower_snake_case(&ty)));
    let field = GrammarNodeField::Node {
        name,
        ty,
        cardinality: Cardinality::Many,
    };
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
        (
            "GenericParamsOwner",
            &["generic_param_list", "where_clause"],
        ),
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
