#[macro_use]
mod generated;

pub use generated::SyntaxKind;

impl From<u16> for SyntaxKind {
    fn from(d: u16) -> SyntaxKind {
        assert!(d <= (SyntaxKind::__LAST as u16));
        unsafe { std::mem::transmute::<u16, SyntaxKind>(d) }
    }
}

impl From<SyntaxKind> for u16 {
    fn from(k: SyntaxKind) -> u16 {
        k as u16
    }
}

impl SyntaxKind {
    pub fn is_trivia(self) -> bool {
        match self {
            SyntaxKind::WHITESPACE | SyntaxKind::COMMENT => true,
            _ => false,
        }
    }
}
