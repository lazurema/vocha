mod deserializer;
mod serializer;

#[cfg(test)]
mod tests;

const OO_BINARY_FILE_MAGIC: &[u8] = b"ooBinaryFile";
const TEXTGRID_OBJECT_CLASS_NAME: &[u8] = b"TextGrid";

pub use deserializer::{DeserializeError, deserialize};
pub use serializer::{SerializeError, serialize};
