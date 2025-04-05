use indexmap::IndexMap;
use std::hash::Hash;

pub trait IdMapKey {
    fn from_usize(id: usize) -> Self;
}

/// Ordered map of keys to values, where keys are constrained to be based on usize.
///
/// Keys are automatically generated in an ever-increasing sequence of positive
/// integers.
///
/// Each item not only has an key, it also has an index! Unlike HashMap, the ordering
/// is stable from run to run. It's also useful because meshes sometimes need to be
/// ordered (e.g. for painter's algorithm or transparency).
///
/// TODO: Consider wrapping each tem in Option<> and hiding this from clients.
/// Would require filtering (and probably a custom iterator) and unwrapping to
/// expose the items. However This would allow [IdMap::move_to_back] and
/// [IdMap::take] to be implemented efficiently.
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

    #[allow(dead_code)]
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
}

impl<K: IdMapKey + Copy + Hash + Eq, V> Default for IdMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
