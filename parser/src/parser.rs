//! See [`Parser`].

use std::cell::Cell;

use crate::lexer::Lexer;
use crate::marker::{CompletedMarker, Marker};
use crate::{
    event::Event,
    ParseError,
    SyntaxKind::{self, EOF, ERROR},
    Token, TokenSet, T,
};

/// `Parser` struct provides the low-level API for
/// navigating through the stream of tokens and
/// constructing the parse tree. The actual parsing
/// happens in the [`grammar`](super::grammar) module.
///
/// However, the result of this `Parser` is not a real
/// tree, but rather a flat stream of events of the form
/// "start expression, consume number literal,
/// finish expression". See `Event` docs for more.
pub(crate) struct Parser<'t> {
    pub(crate) lexer: &'t mut Lexer<'t>,
    pub(crate) events: Vec<Event>,
    pub(crate) steps: Cell<u32>,
}

impl<'t> Parser<'t> {
    pub(super) fn new(lexer: &'t mut Lexer<'t>) -> Parser<'t> {
        Parser {
            lexer,
            events: Vec::new(),
            steps: Cell::new(0),
        }
    }

    pub(crate) fn tokens(&mut self) -> Vec<Token> {
        // self.lexer.bump();
        let mut tokens = self.lexer.all_tokens.clone();
        // tokens.push(Token::new(EOF, 0));
        tokens
    }

    pub(crate) fn finish(self) -> Vec<Event> {
        self.events
    }

    /// Returns the kind of the current token.
    /// If parser has already reached the end of input,
    /// the special `EOF` kind is returned.
    pub(crate) fn current(&self) -> SyntaxKind {
        self.lexer.current().kind
        // self.nth(0)
    }

    // /// Lookahead operation: returns the kind of the next nth
    // /// token.
    // pub(crate) fn nth(&self, n: usize) -> SyntaxKind {
    //     assert!(n <= 3);
    //
    //     let steps = self.steps.get();
    //     assert!(steps <= 10_000_000, "the parser seems stuck");
    //     self.steps.set(steps + 1);
    //
    //     self.lexer.lookahead_nth(n).kind
    // }

    /// Checks if the current token is `kind`.
    pub(crate) fn at(&self, kind: SyntaxKind) -> bool {
        self.nth_at(0, kind)
    }

    pub(crate) fn nth_at(&self, n: usize, kind: SyntaxKind) -> bool {
        match kind {
            // T![-=] => self.at_composite2(n, T![-], T![=]),
            // T![->] => self.at_composite2(n, T![-], T![>]),
            // T![::] => self.at_composite2(n, T![:], T![:]),
            // T![!=] => self.at_composite2(n, T![!], T![=]),
            // T![..] => self.at_composite2(n, T![.], T![.]),
            // T![*=] => self.at_composite2(n, T![*], T![=]),
            // T![/=] => self.at_composite2(n, T![/], T![=]),
            // T![&&] => self.at_composite2(n, T![&], T![&]),
            // T![&=] => self.at_composite2(n, T![&], T![=]),
            // T![%=] => self.at_composite2(n, T![%], T![=]),
            // T![^=] => self.at_composite2(n, T![^], T![=]),
            // T![+=] => self.at_composite2(n, T![+], T![=]),
            // T![<<] => self.at_composite2(n, T![<], T![<]),
            // T![<=] => self.at_composite2(n, T![<], T![=]),
            // T![==] => self.at_composite2(n, T![=], T![=]),
            // T![=>] => self.at_composite2(n, T![=], T![>]),
            // T![>=] => self.at_composite2(n, T![>], T![=]),
            // T![>>] => self.at_composite2(n, T![>], T![>]),
            // T![|=] => self.at_composite2(n, T![|], T![=]),
            // T![||] => self.at_composite2(n, T![|], T![|]),

            // T![...] => self.at_composite3(n, T![.], T![.], T![.]),
            // T![..=] => self.at_composite3(n, T![.], T![.], T![=]),
            // T![<<=] => self.at_composite3(n, T![<], T![<], T![=]),
            // T![>>=] => self.at_composite3(n, T![>], T![>], T![=]),
            _ => self.lexer.lookahead_nth(n).kind == kind,
        }
    }

    /// Consume the next token if `kind` matches.
    pub(crate) fn eat(&mut self, kind: SyntaxKind) -> bool {
        if !self.at(kind) {
            return false;
        }
        // let n_raw_tokens = match kind {
        // | T![||] => 2,

        // T![...] | T![..=] | T![<<=] | T![>>=] => 3,
        // _ => 1,
        // };
        self.do_bump(kind, 1);
        true
    }

    // fn at_composite2(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind) -> bool {
    //     let t1 = self.lexer.lookahead_nth(n);
    //     if t1.kind != k1 || !t1.is_jointed_to_next {
    //         return false;
    //     }
    //     let t2 = self.lexer.lookahead_nth(n + 1);
    //     t2.kind == k2
    // }
    //
    // fn at_composite3(&self, n: usize, k1: SyntaxKind, k2: SyntaxKind, k3: SyntaxKind) -> bool {
    //     let t1 = self.lexer.lookahead_nth(n);
    //     if t1.kind != k1 || !t1.is_jointed_to_next {
    //         return false;
    //     }
    //     let t2 = self.lexer.lookahead_nth(n + 1);
    //     if t2.kind != k2 || !t2.is_jointed_to_next {
    //         return false;
    //     }
    //     let t3 = self.lexer.lookahead_nth(n + 2);
    //     t3.kind == k3
    // }

    /// Checks if the current token is in `kinds`.
    pub(crate) fn at_ts(&self, kinds: TokenSet) -> bool {
        kinds.contains(self.current())
    }

    // /// Checks if the current token is contextual keyword with text `t`.
    // pub(crate) fn at_contextual_kw(&self, kw: &str) -> bool {
    //     self.lexer.is_keyword(kw)
    // }

    /// Starts a new node in the syntax tree. All nodes and tokens
    /// consumed between the `start` and the corresponding `Marker::complete`
    /// belong to the same node.
    pub(crate) fn start(&mut self) -> Marker {
        let pos = self.events.len() as u32;
        self.push_event(Event::tombstone());
        Marker::new(pos)
    }

    /// Consume the next token if `kind` matches.
    pub(crate) fn bump(&mut self, kind: SyntaxKind) {
        assert!(self.eat(kind));
    }

    /// Advances the parser by one token
    pub(crate) fn bump_any(&mut self) {
        // let kind = self.current();
        let kind = self.current();
        if kind == EOF {
            return;
        }
        self.do_bump(kind, 1)
    }

    // /// Advances the parser by one token, remapping its kind.
    // /// This is useful to create contextual keywords from
    // /// identifiers. For example, the lexer creates a `union`
    // /// *identifier* token, but the parser remaps it to the
    // /// `union` keyword, and keyword is what ends up in the
    // /// final tree.
    // pub(crate) fn bump_remap(&mut self, kind: SyntaxKind) {
    //     if self.nth(0) == EOF {
    //         // FIXME: panic!?
    //         return;
    //     }
    //     self.do_bump(kind, 1);
    // }

    /// Emit error with the `message`
    /// FIXME: this should be much more fancy and support
    /// structured errors with spans and notes, like rustc
    /// does.
    pub(crate) fn error<T: Into<String>>(&mut self, message: T) {
        let msg = ParseError(Box::new(message.into()));
        self.push_event(Event::Error { msg })
    }

    /// Consume the next token if it is `kind` or emit an error
    /// otherwise.
    pub(crate) fn expect(&mut self, kind: SyntaxKind) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error(format!("expected {:?}", kind));
        false
    }

    /// Create an error node and consume the next token.
    pub(crate) fn err_and_bump(&mut self, message: &str) {
        match self.current() {
            // L_DOLLAR | R_DOLLAR => {
            //     let m = self.start();
            //     self.error(message);
            //     self.bump_any();
            //     m.complete(self, ERROR);
            // }
            _ => {
                self.err_recover(message, TokenSet::EMPTY);
            }
        }
    }

    pub(crate) fn error_and_skip_until(
        &mut self,
        message: &str,
        end_tokens: TokenSet,
    ) -> CompletedMarker {
        let end_tokens = end_tokens.union(TokenSet::new(&[EOF]));

        let m = self.start();
        self.error(message);
        loop {
            if end_tokens.contains(self.current()) {
                break;
            }
            self.bump_any();
        }
        m.complete(self, ERROR)
    }

    /// Create an error node and consume the next token.
    pub(crate) fn err_recover(&mut self, message: &str, recovery: TokenSet) {
        match self.current() {
            T!['{'] | T!['}'] => {
                self.error(message);
                return;
            }
            _ => (),
        }

        if self.at_ts(recovery) {
            self.error(message);
            return;
        }

        let m = self.start();
        self.error(message);
        self.bump_any();
        m.complete(self, ERROR);
    }

    pub(crate) fn do_bump(&mut self, kind: SyntaxKind, n_raw_tokens: u8) {
        for _ in 0..n_raw_tokens {
            self.lexer.bump();
        }

        self.push_event(Event::Token { kind, n_raw_tokens });
    }

    pub(crate) fn push_event(&mut self, event: Event) {
        self.events.push(event)
    }
}
