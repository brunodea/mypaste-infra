use crate::content::{Content, PasteContent};
use std::collections::HashMap;

/// Cache represents a simple set/get cache system.
pub(crate) trait Cache {
    fn set(&mut self, value: PasteContent);
    fn get(&self, key: String) -> Option<&PasteContent>;
}

/// Cache held in-memory in the same server as the application web server.
pub(crate) struct MemoryCache {
    map: HashMap<String, PasteContent>,
}

impl MemoryCache {
    pub(crate) fn new() -> Self {
        MemoryCache {
            map: HashMap::<String, PasteContent>::new(),
        }
    }
}

impl Cache for MemoryCache {
    fn set(&mut self, value: PasteContent) {
        self.map.insert(value.hash(), value);
    }

    fn get(&self, key: String) -> Option<&PasteContent> {
        self.map.get(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_cache_setget_pastecontent_text_using_hash_key() {
        let mut cache = MemoryCache::new();
        let plain_text = PasteContent::PlainText("ABCDEFG".to_string());
        let plain_text_hash = plain_text.hash();
        cache.set(plain_text);
        assert_eq!(
            Some(&PasteContent::PlainText("ABCDEFG".to_string())),
            cache.get(plain_text_hash)
        );
    }

    #[test]
    fn memory_cache_setget_pastecontent_text_non_existing() {
        let mut cache = MemoryCache::new();
        let plain_text = PasteContent::PlainText("ABCDEFG".to_string());
        cache.set(plain_text);
        assert_eq!(None, cache.get("Non-existing value".to_string()));
    }
}
