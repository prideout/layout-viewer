use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct CellId(pub(crate) usize);

pub struct StringInterner {
    strings: Vec<String>,
    ids: HashMap<String, CellId>,
}

impl StringInterner {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            ids: HashMap::new(),
        }
    }

    pub fn intern(&mut self, s: &str) -> CellId {
        if let Some(&idx) = self.ids.get(s) {
            idx
        } else {
            let idx = CellId(self.strings.len());
            let s = s.to_string();
            self.ids.insert(s.clone(), idx);
            self.strings.push(s);
            idx
        }
    }

    pub fn get(&self, id: CellId) -> &str {
        &self.strings[id.0]
    }

    pub fn get_id(&self, s: &str) -> Option<CellId> {
        self.ids.get(s).copied()
    }
}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}
