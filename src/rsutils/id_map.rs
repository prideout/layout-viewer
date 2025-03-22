use indexmap::IndexMap;
use std::hash::Hash;

pub trait IdMapKey {
    fn from_usize(id: usize) -> Self;
}

pub struct IdMap<K: IdMapKey + Copy + Hash + Eq, V> {
    items: IndexMap<K, V>,
    next_id: usize,
}

impl<K: IdMapKey + Copy + Hash + Eq, V> IdMap<K, V> {
    pub fn new() -> Self {
        Self {
            items: IndexMap::new(),
            next_id: 1,
        }
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn create_id(&mut self) -> K {
        let id = K::from_usize(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn insert(&mut self, value: V) -> K {
        let id = self.create_id();
        self.items.insert(id, value);
        id
    }

    pub fn get(&self, id: &K) -> Option<&V> {
        self.items.get(id)
    }

    pub fn get_mut(&mut self, id: &K) -> Option<&mut V> {
        self.items.get_mut(id)
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.items.values()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.items.values_mut()
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.next_id = 1;
    }
}

impl<K: IdMapKey + Copy + Hash + Eq, V> Default for IdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
