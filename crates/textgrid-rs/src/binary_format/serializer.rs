use std::io::Write;

use snafu::{ResultExt, prelude::*};

use crate::{
    TextGrid, TextGridTier, TextGridTierKind,
    binary_format::{OO_BINARY_FILE_MAGIC, TEXTGRID_OBJECT_CLASS_NAME},
};

#[derive(Debug, Snafu)]
pub enum SerializeError {
    #[snafu(display("I/O error: {}", source))]
    IoError { source: std::io::Error },

    #[snafu(display(
        "String length overflow: allowed maximum length is {}, but actual length is {}",
        allowed,
        actual
    ))]
    StringLengthOverflow { allowed: usize, actual: usize },
}

pub fn serialize<W: Write>(w: &mut W, input: &TextGrid) -> Result<(), SerializeError> {
    w.write_all(OO_BINARY_FILE_MAGIC).context(IoSnafu)?;
    w.write_all(&[TEXTGRID_OBJECT_CLASS_NAME.len() as u8])
        .context(IoSnafu)?;
    w.write_all(TEXTGRID_OBJECT_CLASS_NAME).context(IoSnafu)?;
    write_f64(w, input.xmin)?;
    write_f64(w, input.xmax)?;
    w.write_all(&[0x01]).context(IoSnafu)?; // `<exists>`

    write_i32(w, input.tiers.len() as i32)?;

    for tier in &input.tiers {
        match tier {
            TextGridTier::IntervalTier(interval_tier) => {
                write_tier_kind(w, TextGridTierKind::IntervalTier)?;
                write_text(w, &interval_tier.name)?;
                write_f64(w, interval_tier.xmin)?;
                write_f64(w, interval_tier.xmax)?;
                write_i32(w, interval_tier.intervals.len() as i32)?;

                for interval in &interval_tier.intervals {
                    write_f64(w, interval.xmin)?;
                    write_f64(w, interval.xmax)?;
                    write_text(w, &interval.text)?;
                }
            }
            TextGridTier::TextTier(text_tier) => {
                write_tier_kind(w, TextGridTierKind::TextTier)?;
                write_text(w, &text_tier.name)?;
                write_f64(w, text_tier.xmin)?;
                write_f64(w, text_tier.xmax)?;
                write_i32(w, text_tier.points.len() as i32)?;

                for point in &text_tier.points {
                    write_f64(w, point.number)?;
                    write_text(w, &point.mark)?;
                }
            }
        }
    }

    Ok(())
}

#[inline(always)]
fn write_f64<W: Write>(w: &mut W, value: f64) -> Result<(), SerializeError> {
    w.write_all(&value.to_be_bytes()).context(IoSnafu)
}

#[inline(always)]
fn write_i32<W: Write>(w: &mut W, value: i32) -> Result<(), SerializeError> {
    w.write_all(&value.to_be_bytes()).context(IoSnafu)
}

#[inline(always)]
fn write_i16<W: Write>(w: &mut W, value: i16) -> Result<(), SerializeError> {
    w.write_all(&value.to_be_bytes()).context(IoSnafu)
}

#[inline(always)]
fn write_i8<W: Write>(w: &mut W, value: i8) -> Result<(), SerializeError> {
    w.write_all(&value.to_be_bytes()).context(IoSnafu)
}

fn write_tier_kind<W: Write>(w: &mut W, tier_kind: TextGridTierKind) -> Result<(), SerializeError> {
    let tier_kind_bytes = tier_kind.as_bytes();
    if tier_kind_bytes.len() > i8::MAX as usize {
        return Err(SerializeError::StringLengthOverflow {
            allowed: i8::MAX as usize,
            actual: tier_kind_bytes.len(),
        });
    }
    write_i8(w, tier_kind_bytes.len() as i8)?;
    w.write_all(tier_kind_bytes).context(IoSnafu)
}

fn write_text<W: Write>(w: &mut W, text: &str) -> Result<(), SerializeError> {
    if text.is_ascii() {
        if text.len() > i16::MAX as usize {
            return Err(SerializeError::StringLengthOverflow {
                allowed: i16::MAX as usize,
                actual: text.len(),
            });
        }
        write_i16(w, text.len() as i16)?;
        w.write_all(text.as_bytes()).context(IoSnafu)
    } else {
        write_i16(w, -1 as i16)?;
        let text_utf16be = text
            .encode_utf16()
            .flat_map(|u| u.to_be_bytes())
            .collect::<Vec<u8>>();
        let text_utf16be_len = text_utf16be.len() / 2;
        if text_utf16be_len > i16::MAX as usize {
            return Err(SerializeError::StringLengthOverflow {
                allowed: i16::MAX as usize,
                actual: text_utf16be_len,
            });
        }
        write_i16(w, text_utf16be_len as i16)?;
        w.write_all(&text_utf16be).context(IoSnafu)
    }
}
