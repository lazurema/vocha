use snafu::prelude::*;

use crate::{
    TextGrid, TextGridInterval, TextGridIntervalTier, TextGridPoint, TextGridTextTier,
    TextGridTier, TextGridTierKind,
    text_format::tokenizer::{Token, TokenFlag, TokenKind, TokenizeError, Tokenizer},
};

#[derive(Debug, Snafu)]
pub enum ParseError {
    #[snafu(display("Tokenization error: {}", source))]
    TokenizeError { source: TokenizeError },

    #[snafu(display("Unexpected token kind: expected {}, actual {}", expected, actual))]
    UnexpectedTokenKind {
        expected: TokenKind,
        actual: TokenKind,
    },

    #[snafu(display("Unexpected non-integer number token: {}", actual))]
    UnexpectedNonIntegerNumberToken { actual: String },

    #[snafu(display("Unexpected constant token: expected {}, actual {}", expected, actual))]
    UnexpectedConstantToken {
        expected: &'static str,
        actual: String,
    },

    #[snafu(display("Unexpected end of input: {}", details))]
    UnexpectedEndOfInput { details: String },

    #[snafu(display("Unsupported feature: {}", details))]
    Unsupported { details: String },

    #[snafu(display("Unsupported tier class: {}", name))]
    UnsupportedTierClass { name: String },
}

pub fn parse(input: &str) -> Result<TextGrid, ParseError> {
    let mut tokenizer = Tokenizer::new(input).peekable();

    expect_constant_text_token(&mut tokenizer, input, "ooTextFile")?;
    expect_constant_text_token(&mut tokenizer, input, "TextGrid")?;

    let xmin = expect_number_token(&mut tokenizer, input)?;
    let xmax = expect_number_token(&mut tokenizer, input)?;

    {
        let peeked = tokenizer.peek();
        match peeked {
            Some(Ok(Token::Flag(TokenFlag::Exists))) => {
                tokenizer.next();
            }
            Some(Ok(Token::Flag(TokenFlag::Absent))) | None => {
                return Ok(TextGrid {
                    xmin,
                    xmax,
                    tiers: vec![],
                });
            }
            _ => {}
        }
    }

    let size = expect_integer_number_token(&mut tokenizer, input)?;
    if size < 0 {
        return Err(ParseError::Unsupported {
            details: format!("Negative tier size is not supported: {}", size),
        });
    }
    let size = size as usize;

    let mut tiers: Vec<TextGridTier> = Vec::with_capacity(size);

    for _ in 0..size {
        let class_name = expect_text_token(&mut tokenizer, input)?;
        let tier_kind = TextGridTierKind::from_str(&class_name).ok_or_else(|| {
            ParseError::UnsupportedTierClass {
                name: class_name.clone(),
            }
        })?;
        match tier_kind {
            TextGridTierKind::IntervalTier => {
                let tier = parse_interval_tier(&mut tokenizer, input)?;
                tiers.push(TextGridTier::IntervalTier(tier));
            }
            TextGridTierKind::TextTier => {
                let tier = parse_text_tier(&mut tokenizer, input)?;
                tiers.push(TextGridTier::TextTier(tier));
            }
        }
    }

    Ok(TextGrid { xmin, xmax, tiers })
}

fn parse_interval_tier(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &str,
) -> Result<TextGridIntervalTier, ParseError> {
    let name = expect_text_token(tokenizer, input)?;
    let xmin = expect_number_token(tokenizer, input)?;
    let xmax = expect_number_token(tokenizer, input)?;

    let size = expect_integer_number_token(tokenizer, input)?;
    if size < 0 {
        return Err(ParseError::Unsupported {
            details: format!("Intervals do not support negative size: {}", size),
        });
    }
    let size = size as usize;

    let mut intervals: Vec<TextGridInterval> = Vec::with_capacity(size);

    for _ in 0..size {
        let interval_xmin = expect_number_token(tokenizer, input)?;
        let interval_xmax = expect_number_token(tokenizer, input)?;
        let interval_text = expect_text_token(tokenizer, input)?;

        intervals.push(TextGridInterval {
            xmin: interval_xmin,
            xmax: interval_xmax,
            text: interval_text,
        });
    }

    Ok(TextGridIntervalTier {
        name,
        xmin,
        xmax,
        intervals,
    })
}

fn parse_text_tier(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &str,
) -> Result<TextGridTextTier, ParseError> {
    let name = expect_text_token(tokenizer, input)?;
    let xmin = expect_number_token(tokenizer, input)?;
    let xmax = expect_number_token(tokenizer, input)?;

    let size = expect_integer_number_token(tokenizer, input)?;
    if size < 0 {
        return Err(ParseError::Unsupported {
            details: format!("Points do not support negative size: {}", size),
        });
    }
    let size = size as usize;

    let mut points: Vec<TextGridPoint> = Vec::with_capacity(size);

    for _ in 0..size {
        let number = expect_number_token(tokenizer, input)?;
        let mark = expect_text_token(tokenizer, input)?;

        points.push(TextGridPoint { number, mark });
    }

    Ok(TextGridTextTier {
        name,
        xmin,
        xmax,
        points,
    })
}

fn expect_constant_text_token<'a>(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &'a str,
    expected: &'static str,
) -> Result<(), ParseError> {
    let Some(token) = tokenizer.into_iter().next() else {
        return Err(ParseError::UnexpectedEndOfInput {
            details: format!("Expected constant text token: {}", expected),
        });
    };
    let token = token.context(TokenizeSnafu)?;
    match &token {
        Token::Text(range) => {
            let actual = unescape(input, range);
            if actual == expected {
                Ok(())
            } else {
                Err(ParseError::UnexpectedConstantToken { expected, actual })
            }
        }
        token => {
            return Err(ParseError::UnexpectedTokenKind {
                expected: TokenKind::Text,
                actual: token.kind(),
            });
        }
    }
}

fn expect_text_token<'a>(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &'a str,
) -> Result<String, ParseError> {
    let Some(token) = tokenizer.into_iter().next() else {
        return Err(ParseError::UnexpectedEndOfInput {
            details: "Expected text token".to_string(),
        });
    };
    let token = token.context(TokenizeSnafu)?;
    match &token {
        Token::Text(range) => Ok(unescape(input, range)),
        token => {
            return Err(ParseError::UnexpectedTokenKind {
                expected: TokenKind::Text,
                actual: token.kind(),
            });
        }
    }
}

fn expect_number_token<'a>(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &'a str,
) -> Result<f64, ParseError> {
    let Some(token) = tokenizer.into_iter().next() else {
        return Err(ParseError::UnexpectedEndOfInput {
            details: "Expected number token".to_string(),
        });
    };
    let token = token.context(TokenizeSnafu)?;
    match &token {
        Token::Number(range) => {
            let actual_str = &input[range.clone()];
            if let Ok(actual) = actual_str.parse::<f64>() {
                Ok(actual)
            } else {
                Err(ParseError::Unsupported {
                    details: format!("Unparseable number token: {}", actual_str),
                })
            }
        }
        token => {
            return Err(ParseError::UnexpectedTokenKind {
                expected: TokenKind::Number,
                actual: token.kind(),
            });
        }
    }
}

fn expect_integer_number_token<'a>(
    tokenizer: &mut impl Iterator<Item = Result<Token, TokenizeError>>,
    input: &'a str,
) -> Result<i64, ParseError> {
    let Some(token) = tokenizer.into_iter().next() else {
        return Err(ParseError::UnexpectedEndOfInput {
            details: "Expected integer number token".to_string(),
        });
    };
    let token = token.context(TokenizeSnafu)?;
    match &token {
        Token::Number(range) => {
            let actual_str = &input[range.clone()];
            if migth_be_octal(actual_str) {
                // I don't know how Praat handles octal number literals. And
                // since the original code is in GPL, I can not read it to find
                // out. Therefore, I will just reject potential octal number
                // literals for now to avoid misparsing.
                return Err(ParseError::Unsupported {
                    details: format!("Octal number literals are not supported: {}", actual_str),
                });
            }
            if let Ok(actual) = actual_str.parse::<i64>() {
                Ok(actual)
            } else {
                Err(ParseError::UnexpectedNonIntegerNumberToken {
                    actual: actual_str.to_string(),
                })
            }
        }
        token => {
            return Err(ParseError::UnexpectedTokenKind {
                expected: TokenKind::Number,
                actual: token.kind(),
            });
        }
    }
}

fn migth_be_octal(str: &str) -> bool {
    let index = if matches!(str.as_bytes().first(), Some(b'+') | Some(b'-')) {
        if str.len() == 2 {
            return false;
        }
        1
    } else {
        if str.len() == 1 {
            return false;
        }
        0
    };
    str.as_bytes().get(index) == Some(&b'0')
}

fn unescape(input: &str, range: &std::ops::Range<usize>) -> String {
    input[range.clone()].replace("\"\"", "\"")
}
