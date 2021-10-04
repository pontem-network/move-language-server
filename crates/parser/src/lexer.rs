use crate::SyntaxKind::{self, *};
use rowan::TextSize;

/// A token of Rust source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    /// The kind of token.
    pub kind: SyntaxKind,
    /// The length of the token.
    pub len: TextSize,
}

impl Token {
    pub fn new(kind: SyntaxKind, len: usize) -> Self {
        Token { kind, len: TextSize::from(len as u32) }
    }
}

pub struct Lexer<'t> {
    text: &'t str,
    prev_end: usize,
    cur_start: usize,
    cur_end: usize,
    token: Token,
    pub all_tokens: Vec<Token>,
}

impl<'t> Lexer<'t> {
    pub(crate) fn lookahead_nth(&self, n: usize) -> Token {
        if n == 0 {
            self.current()
        } else if n == 1 {
            self.lookahead()
        } else if n == 2 {
            self.lookahead_2().1
        } else {
            todo!()
        }
    }
}

impl<'t> Lexer<'t> {
    pub fn new(text: &'t str) -> Lexer<'t> {
        let (kind, len) = find_token(text);
        let token = Token::new(kind, len);
        let lexer =
            Lexer { text, prev_end: 0, cur_start: 0, cur_end: len, token, all_tokens: vec![token] };
        lexer
    }

    pub fn current(&self) -> Token {
        self.token
    }

    // pub fn lookahead_nth(&self, n: usize) -> parser::Token {
    //     mk_token(self.curr.1 + n, &self.token_offset_pairs)
    // }

    // pub fn bump(&mut self) {
    //     if self.curr.0.kind == SyntaxKind::EOF {
    //         return;
    //     }
    //
    //     let pos = self.curr.1 + 1;
    //     self.curr = (mk_token(pos, &self.token_offset_pairs), pos);
    // }

    // pub fn is_keyword(&self, kw: &str) -> bool {
    //     self.token_offset_pairs
    //         .get(self.curr.1)
    //         .map(|(token, offset)| &self.text[TextRange::at(*offset, token.len)] == kw)
    //         .unwrap_or(false)
    // }

    pub fn peek(&self) -> Token {
        self.token
    }

    // pub fn content(&self) -> &str {
    //     &self.text[self.cur_start..self.cur_end]
    // }

    // pub fn start_loc(&self) -> usize {
    //     self.cur_start
    // }

    // pub fn previous_end_loc(&self) -> usize {
    //     self.prev_end
    // }

    // Look ahead to the next token after the current one and return it without advancing
    // the state of the lexer.
    pub fn lookahead(&self) -> Token {
        let text = self.text[self.cur_end..].trim_start();
        // let offset = self.text.len() - text.len();
        let (tok, len) = find_token(text);
        Token::new(tok, len)
    }

    // Look ahead to the next two tokens after the current one and return them without advancing
    // the state of the lexer.
    pub fn lookahead_2(&self) -> (Token, Token) {
        let text = &self.text[self.cur_end..];
        // let text = self.text[self.cur_end..].trim_start();
        let offset = self.text.len() - text.len();
        let (first, length) = find_token(text);

        let text2 = &self.text[offset + length..];
        // let text2 = self.text[offset + length..].trim_start();
        // let offset2 = self.text.len() - text2.len();
        let (second, sec_length) = find_token(text2);
        (Token::new(first, length), Token::new(second, sec_length))
    }

    pub fn bump(&mut self) {
        self.prev_end = self.cur_end;
        let text = &self.text[self.cur_end..];
        // let text = self.text[self.cur_end..].trim_start();
        self.cur_start = self.text.len() - text.len();
        let (kind, len) = find_token(text);
        if kind.is_trivia() {
            self.advance_with_token(kind, len);
            self.bump();
            return;
        }
        self.advance_with_token(kind, len);
        // self.cur_end = self.cur_start + len;
        // let token = Token::new(kind, len);
        // self.all_tokens.push(token.clone());
        // self.token = token
    }

    fn advance_with_token(&mut self, kind: SyntaxKind, len: usize) {
        let token = Token::new(kind, len);
        self.cur_end = self.cur_start + len;
        self.all_tokens.push(token);
        self.token = token;
    }

    // Replace the current token. The lexer will always match the longest token,
    // but sometimes the parser will prefer to replace it with a shorter one,
    // e.g., ">" instead of ">>".
    // pub fn replace_token(&mut self, kind: SyntaxKind, len: usize) {
    //     self.token = Token::new(kind, len);
    //     self.cur_end = self.cur_start + len
    // }
}

// Find the next token and its length without changing the state of the lexer.
pub fn find_token(text: &str) -> (SyntaxKind, usize) {
    let c: char = match text.chars().next() {
        Some(next_char) => next_char,
        None => {
            return (EOF, 0);
        }
    };
    let (tok, len) = match c {
        ' ' | '\n' | '\t' => {
            let mut num_space_chars = 1;
            loop {
                let remaining_text = &text[num_space_chars..];
                if !remaining_text.starts_with(" ")
                    && !remaining_text.starts_with("\n")
                    && !remaining_text.starts_with("\t")
                {
                    break;
                }
                num_space_chars += 1;
            }
            (WHITESPACE, num_space_chars)
        }
        '0'..='9' => {
            if text.starts_with("0x") && text.len() > 2 {
                let (tok, hex_len) = get_hex_number(&text[2..]);
                if hex_len == 0 {
                    // Fall back to treating this as a "0" token.
                    (INTEGER_NUMBER, 1)
                } else {
                    (tok, 2 + hex_len)
                }
            } else {
                get_decimal_number(text)
            }
        }
        'A'..='Z' | 'a'..='z' | '_' => {
            let is_hex = text.starts_with("x\"");
            if is_hex || text.starts_with("b\"") {
                let line = &text.lines().next().unwrap()[2..];
                match get_string_len(line) {
                    Some(last_quote) => (BYTE_STRING, 2 + last_quote + 1),
                    None => {
                        todo!()
                        // let loc = make_loc(file, start_offset, start_offset + line.len() + 2);
                        // return todo!()
                        // return Err(diag!(
                        //     if is_hex {
                        //         Syntax::InvalidHexString
                        //     } else {
                        //         Syntax::InvalidByteString
                        //     },
                        //     (loc, "Missing closing quote (\") after byte string")
                        // ));
                    }
                }
            } else {
                let len = get_name_len(text);
                let token = SyntaxKind::from_keyword(&text[..len]).unwrap_or(IDENT);
                (token, len)
            }
        }
        '&' => {
            if text.starts_with("&mut") {
                (AMP_MUT, 4)
            } else if text.starts_with("&&") {
                (AMP_AMP, 2)
            } else {
                (AMP, 1)
            }
        }
        '|' => {
            if text.starts_with("||") {
                (PIPE_PIPE, 2)
            } else {
                (PIPE, 1)
            }
        }
        '=' => {
            if text.starts_with("==>") {
                (EQ_EQ_GT, 3)
            } else if text.starts_with("==") {
                (EQ_EQ, 2)
            } else {
                (EQ, 1)
            }
        }
        '!' => {
            if text.starts_with("!=") {
                (BANG_EQ, 2)
            } else {
                (BANG, 1)
            }
        }
        '<' => {
            if text.starts_with("<==>") {
                (LT_EQ_EQ_GT, 4)
            } else if text.starts_with("<=") {
                (LT_EQ, 2)
            } else if text.starts_with("<<") {
                (LT_LT, 2)
            } else {
                (LT, 1)
            }
        }
        '>' => {
            if text.starts_with(">=") {
                (GT_EQ, 2)
            } else if text.starts_with(">>") {
                (GT_GT, 2)
            } else {
                (GT, 1)
            }
        }
        ':' => {
            if text.starts_with("::") {
                (COLON_COLON, 2)
            } else {
                (COLON, 1)
            }
        }
        '%' => (MOD, 1),
        '(' => (L_PAREN, 1),
        ')' => (R_PAREN, 1),
        '[' => (L_BRACK, 1),
        ']' => (R_BRACK, 1),
        '*' => (STAR, 1),
        '+' => (PLUS, 1),
        ',' => (COMMA, 1),
        '-' => (MINUS, 1),
        '.' => {
            if text.starts_with("..") {
                (DOTDOT, 2)
            } else {
                (DOT, 1)
            }
        }
        '/' => (SLASH, 1),
        ';' => (SEMICOLON, 1),
        '^' => (CARET, 1),
        '{' => (L_BRACE, 1),
        '}' => (R_BRACE, 1),
        '#' => (NUMSIGN, 1),
        '@' => (ATSIGN, 1),
        _ => {
            todo!()
            // let loc = make_loc(file, start_offset, start_offset);
            // return todo!()
            // return Err(diag!(
            //     Syntax::InvalidCharacter,
            //     (loc, format!("Invalid character: '{}'", c))
            // ));
        }
    };

    (tok, len)
}

// Return the length of the substring matching [a-zA-Z0-9_]. Note that
// this does not do any special check for whether the first character
// starts with a number, so the caller is responsible for any additional
// checks on the first character.
fn get_name_len(text: &str) -> usize {
    text.chars()
        .position(|c| !matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '0'..='9'))
        .unwrap_or_else(|| text.len())
}

fn get_decimal_number(text: &str) -> (SyntaxKind, usize) {
    let num_text_len =
        text.chars().position(|c| !matches!(c, '0'..='9')).unwrap_or_else(|| text.len());
    get_number_maybe_with_suffix(text, num_text_len)
}

// Return the length of the substring containing characters in [0-9a-fA-F].
fn get_hex_number(text: &str) -> (SyntaxKind, usize) {
    let num_text_len = text
        .find(|c| !matches!(c, 'a'..='f' | 'A'..='F' | '0'..='9'))
        .unwrap_or_else(|| text.len());
    get_number_maybe_with_suffix(text, num_text_len)
}

// Given the text for a number literal and the length for the characters that match to the number
// portion, checks for a typed suffix.
fn get_number_maybe_with_suffix(text: &str, num_text_len: usize) -> (SyntaxKind, usize) {
    let rest = &text[num_text_len..];
    if rest.starts_with("u8") {
        (INTEGER_NUMBER, num_text_len + 2)
    } else if rest.starts_with("u64") {
        (INTEGER_NUMBER, num_text_len + 3)
    } else if rest.starts_with("u128") {
        (INTEGER_NUMBER, num_text_len + 4)
    } else {
        // No typed suffix
        (INTEGER_NUMBER, num_text_len)
    }
}

// Return the length of the quoted string, or None if there is no closing quote.
fn get_string_len(text: &str) -> Option<usize> {
    let mut pos = 0;
    let mut iter = text.chars();
    while let Some(chr) = iter.next() {
        if chr == '\\' {
            // Skip over the escaped character (e.g., a quote or another backslash)
            if iter.next().is_some() {
                pos += 1;
            }
        } else if chr == '"' {
            return Some(pos);
        }
        pos += 1;
    }
    None
}
