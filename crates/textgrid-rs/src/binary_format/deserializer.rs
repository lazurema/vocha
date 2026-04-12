use snafu::prelude::*;

use crate::{
    TextGrid, TextGridTier, TextGridTierKind,
    binary_format::{OO_BINARY_FILE_MAGIC, TEXTGRID_OBJECT_CLASS_NAME},
};

#[derive(Debug, Snafu)]
pub enum DeserializeError {
    #[snafu(display("This is not an `ooBinaryFile` file."))]
    NotOoBinaryFile,

    #[snafu(display("This is not a `TextGrid` file."))]
    NotTextGridFile,

    #[snafu(display("Unexpected end of input: {}", details))]
    UnexpectedEndOfInput { details: String },

    #[snafu(display("Unsupported feature: {}", details))]
    Unsupported { details: String },

    #[snafu(display("Invalid UTF-8 string: {}", source))]
    InvalidUtf8String { source: std::string::FromUtf8Error },

    #[snafu(display("Invalid UTF-16 string: {}", source))]
    InvalidUtf16String { source: std::string::FromUtf16Error },

    #[snafu(display("Invalid ASCII string: contains non-ASCII bytes"))]
    InvalidAsciiString,

    #[snafu(display("Unsupported tier class: {}", name))]
    UnsupportedTierClass { name: String },
}

pub fn deserialize(input: &[u8]) -> Result<TextGrid, DeserializeError> {
    let mut cursor = 0;

    // `"ooBinaryFile"`
    {
        if !input.starts_with(OO_BINARY_FILE_MAGIC) {
            return Err(DeserializeError::NotOoBinaryFile);
        }
        cursor += OO_BINARY_FILE_MAGIC.len();
    }

    // `"TextGrid"`
    {
        let Some(&n) = input.get(cursor) else {
            return Err(DeserializeError::UnexpectedEndOfInput {
                details: "Expected object class name length byte".to_string(),
            });
        };
        if n != TEXTGRID_OBJECT_CLASS_NAME.len() as u8 {
            return Err(DeserializeError::NotTextGridFile);
        }
        cursor += 1;
        if !input[cursor..].starts_with(TEXTGRID_OBJECT_CLASS_NAME) {
            return Err(DeserializeError::NotTextGridFile);
        }
        cursor += TEXTGRID_OBJECT_CLASS_NAME.len();
    }

    let xmin = expect_f64(input, &mut cursor)?;
    let xmax = expect_f64(input, &mut cursor)?;

    let exists = expect_bool(input, &mut cursor)?;
    if !exists {
        return Ok(TextGrid {
            xmin,
            xmax,
            tiers: vec![],
        });
    }

    let size = expect_i32(input, &mut cursor)?;
    if size < 0 {
        return Err(DeserializeError::Unsupported {
            details: format!("Negative tier size is not supported: {}", size),
        });
    }
    let size = size as usize;

    let mut tiers: Vec<TextGridTier> = Vec::with_capacity(size);

    for _ in 0..size {
        let tier_kind = expect_tier_kind(input, &mut cursor)?;
        let tier_name = expect_string_with_length(input, &mut cursor)?;

        let tier_xmin = expect_f64(input, &mut cursor)?;
        let tier_xmax = expect_f64(input, &mut cursor)?;

        let tier_size = expect_i32(input, &mut cursor)?;
        if tier_size < 0 {
            return Err(DeserializeError::Unsupported {
                details: format!("Items in tiers do not support negative size: {}", tier_size),
            });
        }
        let tier_size = tier_size as usize;

        match tier_kind {
            TextGridTierKind::IntervalTier => {
                let mut intervals = Vec::with_capacity(tier_size);
                for _ in 0..tier_size {
                    let interval_xmin = expect_f64(input, &mut cursor)?;
                    let interval_xmax = expect_f64(input, &mut cursor)?;
                    let interval_text = expect_string_with_length(input, &mut cursor)?;
                    intervals.push(crate::TextGridInterval {
                        xmin: interval_xmin,
                        xmax: interval_xmax,
                        text: interval_text,
                    });
                }
                tiers.push(TextGridTier::IntervalTier(crate::TextGridIntervalTier {
                    name: tier_name,
                    xmin: tier_xmin,
                    xmax: tier_xmax,
                    intervals,
                }));
            }
            TextGridTierKind::TextTier => {
                let mut points = Vec::with_capacity(tier_size);
                for _ in 0..tier_size {
                    let point_number = expect_f64(input, &mut cursor)?;
                    let point_mark = expect_string_with_length(input, &mut cursor)?;
                    points.push(crate::TextGridPoint {
                        number: point_number,
                        mark: point_mark,
                    });
                }
                tiers.push(TextGridTier::TextTier(crate::TextGridTextTier {
                    name: tier_name,
                    xmin: tier_xmin,
                    xmax: tier_xmax,
                    points,
                }));
            }
        }
    }

    Ok(TextGrid { xmin, xmax, tiers })
}

fn expect_bool(input: &[u8], cursor: &mut usize) -> Result<bool, DeserializeError> {
    let Some(&byte) = input.get(*cursor) else {
        return Err(DeserializeError::UnexpectedEndOfInput {
            details: "Expected 1 byte for boolean value".to_string(),
        });
    };
    *cursor += 1;
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(DeserializeError::Unsupported {
            details: format!("Invalid boolean value byte: {}", byte),
        }),
    }
}

fn expect_i8(input: &[u8], cursor: &mut usize) -> Result<i8, DeserializeError> {
    let Some(&byte) = input.get(*cursor) else {
        return Err(DeserializeError::UnexpectedEndOfInput {
            details: "Expected 1 byte for i8 value".to_string(),
        });
    };
    *cursor += 1;
    Ok(byte as i8)
}

fn expect_i16(input: &[u8], cursor: &mut usize) -> Result<i16, DeserializeError> {
    let bytes =
        input
            .get(*cursor..*cursor + 2)
            .ok_or_else(|| DeserializeError::UnexpectedEndOfInput {
                details: "Expected 2 bytes for i16 value".to_string(),
            })?;
    *cursor += 2;
    Ok(i16::from_be_bytes(bytes.try_into().unwrap()))
}

fn expect_i32(input: &[u8], cursor: &mut usize) -> Result<i32, DeserializeError> {
    let bytes =
        input
            .get(*cursor..*cursor + 4)
            .ok_or_else(|| DeserializeError::UnexpectedEndOfInput {
                details: "Expected 4 bytes for i32 value".to_string(),
            })?;
    *cursor += 4;
    Ok(i32::from_be_bytes(bytes.try_into().unwrap()))
}

fn expect_f64(input: &[u8], cursor: &mut usize) -> Result<f64, DeserializeError> {
    let bytes =
        input
            .get(*cursor..*cursor + 8)
            .ok_or_else(|| DeserializeError::UnexpectedEndOfInput {
                details: "Expected 8 bytes for f64 value".to_string(),
            })?;
    *cursor += 8;
    Ok(f64::from_be_bytes(bytes.try_into().unwrap()))
}

fn expect_bytes<'a>(
    input: &'a [u8],
    cursor: &mut usize,
    size: usize,
) -> Result<&'a [u8], DeserializeError> {
    let bytes = input.get(*cursor..*cursor + size).ok_or_else(|| {
        DeserializeError::UnexpectedEndOfInput {
            details: format!("Expected {} bytes", size),
        }
    })?;
    *cursor += size;
    Ok(bytes)
}

fn expect_tier_kind(
    input: &[u8],
    cursor: &mut usize,
) -> Result<TextGridTierKind, DeserializeError> {
    let tier_kind_size = expect_i8(input, cursor)?;
    let tier_kind_bytes = expect_bytes(input, cursor, tier_kind_size as usize)?;

    TextGridTierKind::from_bytes(tier_kind_bytes).context(UnsupportedTierClassSnafu {
        name: String::from_utf8_lossy(tier_kind_bytes).to_string(),
    })
}

fn expect_string_with_length(input: &[u8], cursor: &mut usize) -> Result<String, DeserializeError> {
    fn expect_ascii_string(
        input: &[u8],
        cursor: &mut usize,
        size: usize,
    ) -> Result<String, DeserializeError> {
        let bytes = expect_bytes(input, cursor, size)?;
        if bytes.iter().any(|&b| b > 0x7F) {
            return Err(DeserializeError::InvalidAsciiString);
        }
        Ok(String::from_utf8_lossy(bytes).to_string())
    }
    fn expect_utf16_be_string(
        input: &[u8],
        cursor: &mut usize,
        size: usize,
    ) -> Result<String, DeserializeError> {
        let bytes = expect_bytes(input, cursor, size * 2)?;
        let u16s: Vec<u16> = bytes
            .chunks(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&u16s).map_err(|e| DeserializeError::InvalidUtf16String { source: e })
    }

    let length = expect_i16(input, cursor)?;
    if length >= 0 {
        expect_ascii_string(input, cursor, length as usize)
    } else if length == -1 {
        let utf16_length = expect_i16(input, cursor)?;
        if utf16_length < 0 {
            return Err(DeserializeError::Unsupported {
                details: format!(
                    "Negative UTF-16 string length is not supported: {}",
                    utf16_length
                ),
            });
        }
        expect_utf16_be_string(input, cursor, utf16_length as usize)
    } else {
        Err(DeserializeError::Unsupported {
            details: format!("Negative string length is not supported: {}", length),
        })
    }
}
