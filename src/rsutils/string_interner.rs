#![allow(dead_code)]

use indexmap::IndexMap;

pub struct StringInterner {
    strings: Vec<String>,
    ids: IndexMap<String, usize>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: vec!["null".to_string()],
            ids: IndexMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.ids.get(s) {
            idx
        } else {
            let idx = self.strings.len();
            let s = s.to_string();
            self.ids.insert(s.clone(), idx);
            self.strings.push(s);
            idx
        }
    }

    #[allow(dead_code)]
    pub fn get_id(&self, s: &str) -> Option<usize> {
        self.ids.get(s).copied()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
