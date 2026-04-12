use std::ops::Range;

use snafu::prelude::*;

pub struct Tokenizer<'a> {
    buf: &'a [u8],
    cursor: usize,
}

#[derive(Debug, Clone)]
pub enum Token {
    Number(Range<usize>),
    Text(Range<usize>),
    Flag(TokenFlag),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenFlag {
    Exists,
    Absent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Number,
    Text,
    Flag,
}

impl Token {
    pub fn kind(&self) -> TokenKind {
        match self {
            Token::Number(_) => TokenKind::Number,
            Token::Text(_) => TokenKind::Text,
            Token::Flag(_) => TokenKind::Flag,
        }
    }
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Number => write!(f, "Number"),
            Self::Text => write!(f, "Text"),
            Self::Flag => write!(f, "Flag"),
        }
    }
}

#[derive(Debug, Snafu)]
pub enum TokenizeError {
    #[snafu(display("Unknown flag at range {:?}", range))]
    UnknwonFlag { range: Range<usize> },
    #[snafu(display("Unexpected end of input at range {:?}", range))]
    PartialToken { range: Range<usize> },
}

impl<'a> Tokenizer<'a> {
    pub fn new(buf: &'a str) -> Self {
        Self {
            buf: buf.as_bytes(),
            cursor: 0,
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token, TokenizeError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut can_start_token = true;

        while let Some(&c) = self.buf.get(self.cursor) {
            match c {
                c if can_start_token && c.is_ascii_digit() => {
                    return Some(self.parse_number(false));
                }
                b'+' if can_start_token
                    && self
                        .buf
                        .get(self.cursor + 1)
                        .map_or(false, |&c| c.is_ascii_digit()) =>
                {
                    return Some(self.parse_number(true));
                }
                b'-' if can_start_token
                    && self
                        .buf
                        .get(self.cursor + 1)
                        .map_or(false, |&c| c.is_ascii_digit()) =>
                {
                    return Some(self.parse_number(true));
                }
                b'"' if can_start_token => {
                    return Some(self.parse_text());
                }
                b'<' => {
                    return Some(self.parse_flag());
                }
                b'!' => {
                    self.skip_comments();
                    can_start_token = true;
                }
                b'[' => {
                    if let Err(err) = self.skip_label() {
                        return Some(Err(err));
                    }
                    can_start_token = true;
                }
                b' ' | b'\t' | b'\r' | b'\n' => {
                    self.cursor += 1;
                    can_start_token = true;
                }
                _ => {
                    self.cursor += 1;
                    can_start_token = false;
                }
            }
        }

        None
    }
}

impl<'a> Tokenizer<'a> {
    fn parse_number(&mut self, has_sign: bool) -> Result<Token, TokenizeError> {
        let start = self.cursor;
        self.cursor += 1;
        if has_sign {
            self.cursor += 1;
        }
        let mut is_after_decimal_point = false;

        while let Some(&c) = self.buf.get(self.cursor) {
            if c.is_ascii_digit() {
                self.cursor += 1;
            } else if c == b'.'
                && !is_after_decimal_point
                && self
                    .buf
                    .get(self.cursor + 1)
                    .map_or(false, |&c| c.is_ascii_digit())
            {
                is_after_decimal_point = true;
                self.cursor += 2;
            } else {
                break;
            }
        }

        Ok(Token::Number(start..self.cursor))
    }

    /// Examples:
    /// - `… "Hello World!" …`
    /// - `… "Hello ""World!""" …`
    /// - ```TextGrid
    ///   … "Hello
    ///   World!" …
    ///   ```
    fn parse_text(&mut self) -> Result<Token, TokenizeError> {
        self.cursor += 1;
        let start = self.cursor;

        while let Some(&c) = self.buf.get(self.cursor) {
            if c == b'"' {
                if self.buf.get(self.cursor + 1) == Some(&b'"') {
                    self.cursor += 2;
                } else {
                    let end = self.cursor;
                    self.cursor += 1;
                    return Ok(Token::Text(start..end));
                }
            } else {
                self.cursor += 1;
            }
        }

        Err(TokenizeError::PartialToken {
            range: start - 1..self.buf.len(),
        })
    }

    fn parse_flag(&mut self) -> Result<Token, TokenizeError> {
        self.cursor += 1;

        let Some(offset_close) = self.buf[self.cursor..].iter().position(|&c| c == b'>') else {
            return Err(TokenizeError::PartialToken {
                range: self.cursor..self.buf.len(),
            });
        };

        match &self.buf[self.cursor..self.cursor + offset_close] {
            b"exists" => {
                self.cursor += offset_close + 1;
                Ok(Token::Flag(TokenFlag::Exists))
            }
            b"absent" => {
                self.cursor += offset_close + 1;
                Ok(Token::Flag(TokenFlag::Absent))
            }
            _ => Err(TokenizeError::UnknwonFlag {
                range: self.cursor..self.cursor + offset_close,
            }),
        }
    }

    fn skip_comments(&mut self) {
        self.cursor += 1;

        while let Some(&c) = self.buf.get(self.cursor) {
            self.cursor += 1;
            if matches!(c, b'\r' | b'\n') {
                break;
            }
        }
    }

    fn skip_label(&mut self) -> Result<(), TokenizeError> {
        self.cursor += 1;

        while let Some(&c) = self.buf.get(self.cursor) {
            self.cursor += 1;
            if c == b']' {
                return Ok(());
            }
        }

        Err(TokenizeError::PartialToken {
            range: self.cursor - 1..self.buf.len(),
        })
    }
}
