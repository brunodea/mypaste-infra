use base64;
use md5;

use serde::{Deserialize, Serialize};

/// The Content trait specify a hash function in order to uniquely represent
/// some Content.
pub(crate) trait Content {
    fn hash(&self) -> String;
}

/// Numbers of digits to be used as unique identifier for paste contents.
const HASHLEN: usize = 7usize;

/// PasteContent represents the possible contents in some paste.
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) enum PasteContent {
    PlainText(String),
}

impl Content for PasteContent {
    fn hash(&self) -> String {
        let md5_value = match &self {
            PasteContent::PlainText(txt) => md5::compute(txt),
        };
        let mut base64_value = base64::encode(&format!("{:x}", md5_value));
        base64_value.truncate(HASHLEN);
        base64_value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paste_content_hash_has_size_hashlen() {
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("ABCDEFGHIKLMNOP".to_string())
                .hash()
                .len()
        );
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("A".to_string()).hash().len()
        );
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("123456".to_string()).hash().len()
        );
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("1234567".to_string()).hash().len()
        );
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("12345678".to_string()).hash().len()
        );
        assert_eq!(
            HASHLEN,
            PasteContent::PlainText("".to_string()).hash().len()
        );
    }
}
