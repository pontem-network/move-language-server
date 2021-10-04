use super::*;

pub(crate) fn generate_nodes(kinds: SymbolKindsSrc<'_>, grammar: &GrammarAST) -> String {
    let (node_defs, node_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .nodes
        .iter()
        .map(|node| {
            let grammar_name = format_ident!("{}", node.name);
            let syntax_kind_name = format_ident!("{}", to_upper_snake_case(&node.name));

            // let traits = node.traits.iter().map(|trait_name| {
            //     let trait_name = format_ident!("{}", trait_name);
            //     quote!(impl ast::#trait_name for #grammar_name {})
            // });

            let methods = node.fields.iter().map(|field| {
                let method_name = field.method_name();
                let field_type = field.ty();

                if field.is_many() {
                    quote! {
                        pub fn #method_name(&self) -> AstChildren<#field_type> {
                            support::children(&self.syntax)
                        }
                    }
                } else if let Some(token_kind) = field.token_kind() {
                    quote! {
                        pub fn #method_name(&self) -> Option<#field_type> {
                            support::token(&self.syntax, #token_kind)
                        }
                    }
                } else {
                    quote! {
                        pub fn #method_name(&self) -> Option<#field_type> {
                            support::child(&self.syntax)
                        }
                    }
                }
            });
            let node_defs = quote! {
                #[pretty_doc_comment_placeholder_workaround]
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub struct #grammar_name {
                    pub(crate) syntax: SyntaxNode,
                }

                // #(#traits)*

                impl #grammar_name {
                    #(#methods)*
                }
            };
            let node_boilerplace_impls = quote! {
                impl AstNode for #grammar_name {
                    fn can_cast(kind: SyntaxKind) -> bool {
                        kind == #syntax_kind_name
                    }
                    fn cast(syntax: SyntaxNode) -> Option<Self> {
                        if Self::can_cast(syntax.kind()) { Some(Self { syntax }) } else { None }
                    }
                    fn syntax(&self) -> &SyntaxNode { &self.syntax }
                }
            };
            (node_defs, node_boilerplace_impls)
        })
        .unzip();

    let (enum_defs, enum_boilerplate_impls): (Vec<_>, Vec<_>) = grammar
        .enums
        .iter()
        .map(|en| {
            let variants: Vec<_> = en.variants.iter().map(|var| format_ident!("{}", var)).collect();
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
                            matches!(kind, #(#kinds)|*)
                            // match kind {
                            //     #(#kinds)|* => true,
                            //     _ => false,
                            // }
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

            let enum_def = quote! {
                #[pretty_doc_comment_placeholder_workaround]
                #[derive(Debug, Clone, PartialEq, Eq, Hash)]
                pub enum #name {
                    #(#variants(#variants),)*
                }

                #(#traits)*
            };
            let enum_boilerplate_impl = quote! {
                #(
                    impl From<#variants> for #name {
                        fn from(node: #variants) -> #name {
                            #name::#variants(node)
                        }
                    }
                )*
                #ast_node
            };
            (enum_def, enum_boilerplate_impl)
        })
        .unzip();

    let enum_names = grammar.enums.iter().map(|it| &it.name);
    let node_names = grammar.nodes.iter().map(|it| &it.name);

    let display_impls =
        enum_names.chain(node_names.clone()).map(|it| format_ident!("{}", it)).map(|name| {
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
        use crate::ast::{self, AstNode, AstChildren, support};
        use parser::T;

        #(#node_defs)*
        #(#enum_defs)*
        #(#node_boilerplate_impls)*
        #(#enum_boilerplate_impls)*
        #(#display_impls)*
    };

    let ast = ast.to_string().replace("T ! [", "T![");

    let mut res = String::with_capacity(ast.len() * 2);

    let mut docs =
        grammar.nodes.iter().map(|it| &it.doc).chain(grammar.enums.iter().map(|it| &it.doc));

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
