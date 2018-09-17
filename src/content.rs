use base64;
use md5;

pub(crate) trait Content {
    fn hash(&self) -> String;
}

const HASHLEN: usize = 7usize;

#[derive(Debug, Serialize, Deserialize, Clone)]
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
