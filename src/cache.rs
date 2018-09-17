use content::{Content, PasteContent};
use std::collections::HashMap;

pub(crate) trait Cache {
    fn set(&mut self, value: PasteContent);
    fn get(&self, key: String) -> Option<&PasteContent>;
}

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
