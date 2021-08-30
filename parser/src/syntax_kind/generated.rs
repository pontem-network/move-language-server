#![allow(bad_style, missing_docs, unreachable_pub)]
#[doc = r" The kind of syntax node, e.g. `IDENT`, `USE_KW`, or `STRUCT`."]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u16)]
pub enum SyntaxKind {
    #[doc(hidden)]
    TOMBSTONE,
    #[doc(hidden)]
    EOF,
    COLON,
    COLON_COLON,
    SEMICOLON,
    COMMA,
    L_PAREN,
    R_PAREN,
    L_BRACE,
    R_BRACE,
    L_BRACK,
    R_BRACK,
    PLUS,
    MINUS,
    STAR,
    SLASH,
    MOD,
    NUMSIGN,
    ATSIGN,
    DOT,
    DOTDOT,
    AMP,
    AMP_AMP,
    AMP_MUT,
    CARET,
    PIPE,
    PIPE_PIPE,
    BANG,
    BANG_EQ,
    EQ,
    EQ_EQ,
    EQ_EQ_GT,
    GT,
    GT_GT,
    GT_EQ,
    LT,
    LT_LT,
    LT_EQ,
    LT_EQ_EQ_GT,
    STRUCT_KW,
    MODULE_KW,
    CONST_KW,
    USE_KW,
    AS_KW,
    LET_KW,
    MUT_KW,
    RETURN_KW,
    FUN_KW,
    TRUE_KW,
    FALSE_KW,
    MOVE_KW,
    COPY_KW,
    WHILE_KW,
    IF_KW,
    ELSE_KW,
    BREAK_KW,
    CONTINUE_KW,
    INTEGER_NUMBER,
    BYTE_STRING,
    HEX_STRING,
    ERROR,
    IDENT,
    WHITESPACE,
    COMMENT,
    SOURCE_FILE,
    EXPR,
    LITERAL,
    BIN_EXPR,
    PREFIX_EXPR,
    PATH_EXPR,
    PAREN_EXPR,
    EXPR_STMT,
    #[doc(hidden)]
    __LAST,
}
use self::SyntaxKind::*;
impl SyntaxKind {
    pub fn is_keyword(self) -> bool {
        match self {
            STRUCT_KW | MODULE_KW | CONST_KW | USE_KW | AS_KW | LET_KW | MUT_KW | RETURN_KW
            | FUN_KW | TRUE_KW | FALSE_KW | MOVE_KW | COPY_KW | WHILE_KW | IF_KW | ELSE_KW
            | BREAK_KW | CONTINUE_KW => true,
            _ => false,
        }
    }
    pub fn is_punct(self) -> bool {
        match self {
            COLON | COLON_COLON | SEMICOLON | COMMA | L_PAREN | R_PAREN | L_BRACE | R_BRACE
            | L_BRACK | R_BRACK | PLUS | MINUS | STAR | SLASH | MOD | NUMSIGN | ATSIGN | DOT
            | DOTDOT | AMP | AMP_AMP | AMP_MUT | CARET | PIPE | PIPE_PIPE | BANG | BANG_EQ | EQ
            | EQ_EQ | EQ_EQ_GT | GT | GT_GT | GT_EQ | LT | LT_LT | LT_EQ | LT_EQ_EQ_GT => true,
            _ => false,
        }
    }
    pub fn is_literal(self) -> bool {
        match self {
            INTEGER_NUMBER | BYTE_STRING | HEX_STRING => true,
            _ => false,
        }
    }
    pub fn from_keyword(ident: &str) -> Option<SyntaxKind> {
        let kw = match ident {
            "struct" => STRUCT_KW,
            "module" => MODULE_KW,
            "const" => CONST_KW,
            "use" => USE_KW,
            "as" => AS_KW,
            "let" => LET_KW,
            "mut" => MUT_KW,
            "return" => RETURN_KW,
            "fun" => FUN_KW,
            "true" => TRUE_KW,
            "false" => FALSE_KW,
            "move" => MOVE_KW,
            "copy" => COPY_KW,
            "while" => WHILE_KW,
            "if" => IF_KW,
            "else" => ELSE_KW,
            "break" => BREAK_KW,
            "continue" => CONTINUE_KW,
            _ => return None,
        };
        Some(kw)
    }
    pub fn from_char(c: char) -> Option<SyntaxKind> {
        let tok = match c {
            ':' => COLON,
            ';' => SEMICOLON,
            ',' => COMMA,
            '(' => L_PAREN,
            ')' => R_PAREN,
            '{' => L_BRACE,
            '}' => R_BRACE,
            '[' => L_BRACK,
            ']' => R_BRACK,
            '+' => PLUS,
            '-' => MINUS,
            '*' => STAR,
            '/' => SLASH,
            '%' => MOD,
            '#' => NUMSIGN,
            '@' => ATSIGN,
            '.' => DOT,
            '&' => AMP,
            '^' => CARET,
            '|' => PIPE,
            '!' => BANG,
            '=' => EQ,
            '>' => GT,
            '<' => LT,
            _ => return None,
        };
        Some(tok)
    }
}
#[macro_export]
macro_rules ! T { [:] => { $ crate :: SyntaxKind :: COLON } ; [::] => { $ crate :: SyntaxKind :: COLON_COLON } ; [;] => { $ crate :: SyntaxKind :: SEMICOLON } ; [,] => { $ crate :: SyntaxKind :: COMMA } ; ['('] => { $ crate :: SyntaxKind :: L_PAREN } ; [')'] => { $ crate :: SyntaxKind :: R_PAREN } ; ['{'] => { $ crate :: SyntaxKind :: L_BRACE } ; ['}'] => { $ crate :: SyntaxKind :: R_BRACE } ; ['['] => { $ crate :: SyntaxKind :: L_BRACK } ; [']'] => { $ crate :: SyntaxKind :: R_BRACK } ; [+] => { $ crate :: SyntaxKind :: PLUS } ; [-] => { $ crate :: SyntaxKind :: MINUS } ; [*] => { $ crate :: SyntaxKind :: STAR } ; [/] => { $ crate :: SyntaxKind :: SLASH } ; [%] => { $ crate :: SyntaxKind :: MOD } ; [#] => { $ crate :: SyntaxKind :: NUMSIGN } ; [@] => { $ crate :: SyntaxKind :: ATSIGN } ; [.] => { $ crate :: SyntaxKind :: DOT } ; [..] => { $ crate :: SyntaxKind :: DOTDOT } ; [&] => { $ crate :: SyntaxKind :: AMP } ; [&&] => { $ crate :: SyntaxKind :: AMP_AMP } ; [&mut] => { $ crate :: SyntaxKind :: AMP_MUT } ; [^] => { $ crate :: SyntaxKind :: CARET } ; [|] => { $ crate :: SyntaxKind :: PIPE } ; [||] => { $ crate :: SyntaxKind :: PIPE_PIPE } ; [!] => { $ crate :: SyntaxKind :: BANG } ; [!=] => { $ crate :: SyntaxKind :: BANG_EQ } ; [=] => { $ crate :: SyntaxKind :: EQ } ; [==] => { $ crate :: SyntaxKind :: EQ_EQ } ; [==>] => { $ crate :: SyntaxKind :: EQ_EQ_GT } ; [>] => { $ crate :: SyntaxKind :: GT } ; [>>] => { $ crate :: SyntaxKind :: GT_GT } ; [>=] => { $ crate :: SyntaxKind :: GT_EQ } ; [<] => { $ crate :: SyntaxKind :: LT } ; [<<] => { $ crate :: SyntaxKind :: LT_LT } ; [<=] => { $ crate :: SyntaxKind :: LT_EQ } ; [<==>] => { $ crate :: SyntaxKind :: LT_EQ_EQ_GT } ; [struct] => { $ crate :: SyntaxKind :: STRUCT_KW } ; [module] => { $ crate :: SyntaxKind :: MODULE_KW } ; [const] => { $ crate :: SyntaxKind :: CONST_KW } ; [use] => { $ crate :: SyntaxKind :: USE_KW } ; [as] => { $ crate :: SyntaxKind :: AS_KW } ; [let] => { $ crate :: SyntaxKind :: LET_KW } ; [mut] => { $ crate :: SyntaxKind :: MUT_KW } ; [return] => { $ crate :: SyntaxKind :: RETURN_KW } ; [fun] => { $ crate :: SyntaxKind :: FUN_KW } ; [true] => { $ crate :: SyntaxKind :: TRUE_KW } ; [false] => { $ crate :: SyntaxKind :: FALSE_KW } ; [move] => { $ crate :: SyntaxKind :: MOVE_KW } ; [copy] => { $ crate :: SyntaxKind :: COPY_KW } ; [while] => { $ crate :: SyntaxKind :: WHILE_KW } ; [if] => { $ crate :: SyntaxKind :: IF_KW } ; [else] => { $ crate :: SyntaxKind :: ELSE_KW } ; [break] => { $ crate :: SyntaxKind :: BREAK_KW } ; [continue] => { $ crate :: SyntaxKind :: CONTINUE_KW } ; [ident] => { $ crate :: SyntaxKind :: IDENT } ; [shebang] => { $ crate :: SyntaxKind :: SHEBANG } ; }
