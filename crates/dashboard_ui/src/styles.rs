use std::{fmt::Display, panic};

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TokenKind {
    Bg,
    Text,
    Font,
    Padding,
    Px,
    Py,
    Pt,
    Pr,
    Pl,
    Pb,
    Margin,
    Mx,
    My,
    Rounded,
    Assign,

    Identifier,
    // whitespace
    Whitespace,
    Newline,
    Eof,
    Error,
}

#[derive(Debug, Clone, Copy)]
pub struct Token<'a> {
    pub kind: TokenKind,
    pub text: &'a str,
    pub span: Span,
}

pub struct Lexer<'a> {
    pub input: &'a [u8],
    pub pos: usize,
    pub len: usize,
}

fn is_alpha(char: u8) -> bool {
    return char.is_ascii_alphanumeric();
}

fn is_whitespace(char: u8) -> bool {
    return char.is_ascii_whitespace();
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let token = match self {
            TokenKind::Bg => "bg",
            TokenKind::Text => "text",
            TokenKind::Font => "font",
            TokenKind::Padding => "padding",
            TokenKind::Px => "px",
            TokenKind::Py => "py",
            TokenKind::Pt => "pt",
            TokenKind::Pr => "pr",
            TokenKind::Pl => "pl",
            TokenKind::Pb => "pb",
            TokenKind::Margin => "margin",
            TokenKind::Mx => "mx",
            TokenKind::My => "my",
            TokenKind::Rounded => "rounded",
            TokenKind::Assign => "-",
            TokenKind::Identifier => "identifier",
            TokenKind::Whitespace => "whitespace",
            TokenKind::Newline => "newline",
            TokenKind::Eof => "eof",
            TokenKind::Error => "error",
        };

        write!(f, "{token}")
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.kind == TokenKind::Identifier {
            write!(f, "{}", self.text)
        } else if self.kind == TokenKind::Error {
            write!(f, "error")
        } else if self.kind == TokenKind::Eof {
            write!(f, "eof")
        } else {
            write!(f, "'{}'", self.text)
        }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0:{}", self.start)
    }
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> u8 {
        if self.pos == self.len {
            return 0;
        }

        return self.input[self.pos];
    }

    fn bump(&mut self) {
        self.pos = self.len.min(self.pos + 1)
    }

    fn bump_peek(&mut self) -> u8 {
        self.bump();
        return self.peek();
    }

    fn read_kind(&mut self) -> TokenKind {
        let mut c = self.peek();
        if c == 0 {
            return TokenKind::Eof;
        }

        match c {
            b'b' => {
                self.bump();
                c = self.peek();
                if c == b'g' {
                    self.bump();
                    TokenKind::Bg
                } else {
                    TokenKind::Identifier
                }
            }
            b'm' => {
                self.bump();
                TokenKind::Margin
            }
            b'p' => {
                self.bump();
                TokenKind::Padding
            }
            b'-' => {
                self.bump();
                TokenKind::Assign
            }
            mut c if is_alpha(c) => {
                let start = self.pos;
                loop {
                    c = self.bump_peek();

                    if !is_alpha(c) {
                        break;
                    }
                }

                let value = str::from_utf8(&self.input[start..self.pos]).unwrap();
                match value {
                    "bg" => TokenKind::Bg,
                    "text" => TokenKind::Text,
                    "font" => TokenKind::Font,
                    "px" => TokenKind::Px,
                    "py" => TokenKind::Py,
                    "pt" => TokenKind::Pt,
                    "pr" => TokenKind::Pr,
                    "pb" => TokenKind::Pb,
                    "mx" => TokenKind::Mx,
                    "my" => TokenKind::My,
                    "rounded" => TokenKind::Rounded,

                    _ => TokenKind::Identifier,
                }
            }
            c if is_whitespace(c) => {
                self.bump();
                TokenKind::Whitespace
            }
            _ => {
                self.bump();
                TokenKind::Error
            }
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        let mut start = self.pos;
        let mut kind = self.read_kind();

        while kind == TokenKind::Whitespace {
            start = self.pos;
            kind = self.read_kind();
        }

        let span = Span {
            start,
            end: self.pos,
        };

        if kind == TokenKind::Error {
            let value = str::from_utf8(&self.input[start..self.pos]).unwrap();
            panic!("cannot lex {value} at {span}")
        }

        let text = str::from_utf8(&self.input[start..self.pos]).unwrap();
        return Token { kind, text, span };
    }
}

pub struct Merger<'a> {
    pub input_lexer: Lexer<'a>,
    pub merge_lexer: Lexer<'a>,

    pub input_current_token: Token<'a>,
    pub input_current_kind: TokenKind,
    pub input_lookahead_token: Token<'a>,
    pub input_lookahead_kind: TokenKind,

    pub merge_current_token: Token<'a>,
    pub merge_current_kind: TokenKind,
    pub merge_lookahead_token: Token<'a>,
    pub merge_lookahead_kind: TokenKind,
}

impl<'a> Merger<'a> {
    pub fn new(input: &'a [u8], merge: &'a [u8]) -> Self {
        let mut input_lexer = Lexer {
            pos: 0,
            input,
            len: input.len(),
        };
        let mut merge_lexer = Lexer {
            pos: 0,
            input: merge,
            len: merge.len(),
        };

        let input_current_token = input_lexer.next_token();
        let input_current_kind = input_current_token.kind;
        let input_lookahead_token = input_lexer.next_token();
        let input_lookahead_kind = input_lookahead_token.kind;

        let merge_current_token = merge_lexer.next_token();
        let merge_current_kind = merge_current_token.kind;
        let merge_lookahead_token = merge_lexer.next_token();
        let merge_lookahead_kind = merge_lookahead_token.kind;

        Self {
            input_lexer,
            merge_lexer,

            input_current_token,
            input_current_kind,
            input_lookahead_token,
            input_lookahead_kind,

            merge_current_token,
            merge_current_kind,
            merge_lookahead_token,
            merge_lookahead_kind,
        }
    }

    fn input_current_is(&self, kind: TokenKind) -> bool {
        return self.input_current_kind == kind;
    }

    fn merge_current_is(&self, kind: TokenKind) -> bool {
        return self.merge_current_kind == kind;
    }

    fn consume(&mut self) -> (Token<'a>, Token<'a>) {
        let input_old_token = self.input_current_token;
        let merge_old_token = self.merge_current_token;

        self.input_current_token = self.input_lexer.next_token();
        self.input_current_kind = self.input_current_token.kind;
        self.input_lookahead_token = self.input_lexer.next_token();
        self.input_lookahead_kind = self.input_lookahead_token.kind;

        self.merge_current_token = self.merge_lexer.next_token();
        self.merge_current_kind = self.merge_current_token.kind;
        self.merge_lookahead_token = self.merge_lexer.next_token();
        self.merge_lookahead_kind = self.merge_lookahead_token.kind;

        return (input_old_token, merge_old_token);
    }

    fn input_expect(&mut self, kind: TokenKind) {
        if !self.input_current_is(kind) {
            panic!("expected {kind}, but got {kind}")
        }
    }

    fn merge_expect(&mut self, kind: TokenKind) {
        if !self.merge_current_is(kind) {
            panic!("expected {kind}, but got {kind}")
        }
    }

    pub fn merge_styles(&mut self) -> String {
        loop {
            let (input_token, merge_token) = self.consume();
            tracing::info!(input_token = ?input_token, merge_token = ?merge_token, "lexed token");

            if input_token.kind == TokenKind::Eof || merge_token.kind == TokenKind::Eof {
                break;
            }
        }

        return String::new();
    }
}

pub fn merge_styles(input: &[u8], merge: &[u8]) -> String {
    let mut merger = Merger::new(input, merge);
    return merger.merge_styles();
}
