use snafu::{ResultExt, prelude::*};

pub mod binary_format;
pub mod text_format;

#[cfg(test)]
static TEST_FIXTURES_DIR: include_dir::Dir =
    include_dir::include_dir!("$CARGO_MANIFEST_DIR/test-fixtures");

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextGrid {
    pub xmin: f64,
    pub xmax: f64,
    pub tiers: Vec<TextGridTier>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TextGridTier {
    IntervalTier(TextGridIntervalTier),
    TextTier(TextGridTextTier),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextGridTierKind {
    IntervalTier,
    TextTier,
}

impl TextGridTier {
    pub fn text_grid_tier_kind(&self) -> TextGridTierKind {
        match self {
            TextGridTier::IntervalTier(_) => TextGridTierKind::IntervalTier,
            TextGridTier::TextTier(_) => TextGridTierKind::TextTier,
        }
    }
}

impl TextGridTierKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "IntervalTier" => Some(TextGridTierKind::IntervalTier),
            "TextTier" => Some(TextGridTierKind::TextTier),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TextGridTierKind::IntervalTier => "IntervalTier",
            TextGridTierKind::TextTier => "TextTier",
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        match bytes {
            b"IntervalTier" => Some(TextGridTierKind::IntervalTier),
            b"TextTier" => Some(TextGridTierKind::TextTier),
            _ => None,
        }
    }

    pub fn as_bytes(&self) -> &'static [u8] {
        match self {
            TextGridTierKind::IntervalTier => b"IntervalTier",
            TextGridTierKind::TextTier => b"TextTier",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextGridIntervalTier {
    pub name: String,
    pub xmin: f64,
    pub xmax: f64,
    pub intervals: Vec<TextGridInterval>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextGridInterval {
    pub xmin: f64,
    pub xmax: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextGridTextTier {
    pub name: String,
    pub xmin: f64,
    pub xmax: f64,
    pub points: Vec<TextGridPoint>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TextGridPoint {
    pub number: f64,
    pub mark: String,
}

#[derive(Debug, Snafu)]
pub enum TextFormatParseErrorEx {
    TextFormatParseError { source: text_format::ParseError },
    TextFormatFromUtf16Error { source: std::string::FromUtf16Error },
    TextFormatUtf8Error { source: std::str::Utf8Error },
}

impl TextGrid {
    /// Parses a TextGrid from a UTF-8 encoded string slice.
    pub fn parse_text_format_utf8(input: &str) -> Result<Self, text_format::ParseError> {
        text_format::parse(input)
    }

    /// Parses a TextGrid from a byte slice. The input is expected to be either
    /// UTF-8 (without BOM) or UTF-16 encoded.
    pub fn parse_text_format(input: &[u8]) -> Result<Self, TextFormatParseErrorEx> {
        let result = if input.starts_with(&[0xFF, 0xFE]) {
            // UTF-16 LE with BOM
            let u16s = &input[2..]
                .chunks(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<u16>>();
            Self::parse_text_format_utf8(
                &String::from_utf16(u16s).context(TextFormatFromUtf16Snafu)?,
            )
        } else if input.starts_with(&[0xFE, 0xFF]) {
            // UTF-16 BE with BOM
            let u16s = &input[2..]
                .chunks(2)
                .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                .collect::<Vec<u16>>();
            Self::parse_text_format_utf8(
                &String::from_utf16(u16s).context(TextFormatFromUtf16Snafu)?,
            )
        } else {
            // Assume UTF-8 without BOM
            Self::parse_text_format_utf8(std::str::from_utf8(input).context(TextFormatUtf8Snafu)?)
        };

        result.context(TextFormatParseSnafu)
    }

    /// Stringifies the TextGrid into a relatively long format and writes the
    /// result into the given writer.
    pub fn stringify_long_with_writer<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        text_format::stringify_long(w, self)
    }

    /// Stringifies the TextGrid into a relatively short format and writes the
    /// result into the given writer.
    pub fn stringify_short_with_writer<W: std::io::Write>(&self, w: &mut W) -> std::io::Result<()> {
        text_format::stringify_short(w, self)
    }

    /// Stringifies the TextGrid into a relatively long format and returns the
    /// resulting string.
    pub fn stringify_long(&self) -> std::io::Result<String> {
        let mut stringified = Vec::new();
        self.stringify_long_with_writer(&mut stringified)?;
        Ok(String::from_utf8_lossy(&stringified).into_owned())
    }

    /// Stringifies the TextGrid into a relatively short format and returns the
    /// resulting string.
    pub fn stringify_short(&self) -> std::io::Result<String> {
        let mut stringified = Vec::new();
        self.stringify_short_with_writer(&mut stringified)?;
        Ok(String::from_utf8_lossy(&stringified).into_owned())
    }

    /// Deserializes a TextGrid from the binary format.
    pub fn deserialize_binary_format(
        input: &[u8],
    ) -> Result<Self, binary_format::DeserializeError> {
        binary_format::deserialize(input)
    }

    /// Serializes the TextGrid into the binary format and writes the result
    /// into the given writer.
    pub fn serialize_binary_format_with_writer<W: std::io::Write>(
        &self,
        w: &mut W,
    ) -> Result<(), binary_format::SerializeError> {
        binary_format::serialize(w, self)
    }

    /// Serializes the TextGrid into the binary format and returns the resulting
    ///  bytes.
    pub fn serialize_binary_format(&self) -> Result<Vec<u8>, binary_format::SerializeError> {
        let mut serialized = Vec::new();
        self.serialize_binary_format_with_writer(&mut serialized)?;
        Ok(serialized)
    }
}
