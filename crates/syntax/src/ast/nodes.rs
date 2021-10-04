use crate::ast::{self, support, AstChildren, AstNode};
use crate::syntax_node::{SyntaxNode, SyntaxToken};
use crate::SyntaxKind::{self, *};
use parser::T;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceFile {
    pub(crate) syntax: SyntaxNode,
}
impl SourceFile {
    pub fn script_defs(&self) -> AstChildren<ScriptDef> { support::children(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ScriptDef {
    pub(crate) syntax: SyntaxNode,
}
impl ScriptDef {
    pub fn l_brace_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn function_defs(&self) -> AstChildren<FunctionDef> { support::children(&self.syntax) }
    pub fn r_brace_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionDef {
    pub(crate) syntax: SyntaxNode,
}
impl FunctionDef {
    pub fn fun_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![fun]) }
    pub fn ident_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![ident]) }
    pub fn l_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['(']) }
    pub fn r_paren_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![')']) }
    pub fn l_brace_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['{']) }
    pub fn expr_stmts(&self) -> AstChildren<ExprStmt> { support::children(&self.syntax) }
    pub fn r_brace_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T!['}']) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExprStmt {
    pub(crate) syntax: SyntaxNode,
}
impl ExprStmt {
    pub fn expr(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn semicolon_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![;]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinExpr {
    pub(crate) syntax: SyntaxNode,
}
impl BinExpr {
    pub fn lhs(&self) -> Option<Expr> { support::child(&self.syntax) }
    pub fn plus_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![+]) }
    pub fn minus_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![-]) }
    pub fn star_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![*]) }
    pub fn slash_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![/]) }
    pub fn mod_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![%]) }
    pub fn rhs(&self) -> Option<Expr> { support::child(&self.syntax) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Literal {
    pub(crate) syntax: SyntaxNode,
}
impl Literal {
    pub fn true_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![true]) }
    pub fn false_token(&self) -> Option<SyntaxToken> { support::token(&self.syntax, T![false]) }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    BinExpr(BinExpr),
    Literal(Literal),
}
impl AstNode for SourceFile {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SOURCE_FILE }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ScriptDef {
    fn can_cast(kind: SyntaxKind) -> bool { kind == SCRIPT_DEF }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for FunctionDef {
    fn can_cast(kind: SyntaxKind) -> bool { kind == FUNCTION_DEF }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for ExprStmt {
    fn can_cast(kind: SyntaxKind) -> bool { kind == EXPR_STMT }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for BinExpr {
    fn can_cast(kind: SyntaxKind) -> bool { kind == BIN_EXPR }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl AstNode for Literal {
    fn can_cast(kind: SyntaxKind) -> bool { kind == LITERAL }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        if Self::can_cast(syntax.kind()) {
            Some(Self { syntax })
        } else {
            None
        }
    }
    fn syntax(&self) -> &SyntaxNode { &self.syntax }
}
impl From<BinExpr> for Expr {
    fn from(node: BinExpr) -> Expr { Expr::BinExpr(node) }
}
impl From<Literal> for Expr {
    fn from(node: Literal) -> Expr { Expr::Literal(node) }
}
impl AstNode for Expr {
    fn can_cast(kind: SyntaxKind) -> bool { matches!(kind, BIN_EXPR | LITERAL) }
    fn cast(syntax: SyntaxNode) -> Option<Self> {
        let res = match syntax.kind() {
            BIN_EXPR => Expr::BinExpr(BinExpr { syntax }),
            LITERAL => Expr::Literal(Literal { syntax }),
            _ => return None,
        };
        Some(res)
    }
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Expr::BinExpr(it) => &it.syntax,
            Expr::Literal(it) => &it.syntax,
        }
    }
}
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for SourceFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ScriptDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for FunctionDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for ExprStmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for BinExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self.syntax(), f)
    }
}
