//! Storage history for implementation-owned hash tables. This is private copy
//! bookkeeping, independent of gameplay equality, serde and attempt budgets.

use std::{
    borrow::Borrow,
    collections::{hash_map, HashMap},
    fmt,
    hash::{BuildHasher, Hash, RandomState},
    ops::Deref,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Insertion/reservation credit survives tombstones and clear. Ordinary writes
/// saturate; an opted-in clone must reject an unknown/overflowed bound.
#[derive(Clone, Copy, Default)]
pub(crate) struct MapCloneHistory {
    allocation_credit: usize,
}

impl MapCloneHistory {
    /// Only for a source-proven fresh or insert/clear-only table. Never seed
    /// arbitrary previously deleted/retained tables from their current capacity.
    pub(crate) fn from_tombstone_free_capacity(capacity: usize) -> Self {
        Self {
            allocation_credit: capacity,
        }
    }

    /// Equality is only an insertion-generation check within an already proven
    /// native clone lineage. It is never a proof for an arbitrary public map.
    pub(crate) fn same_unsaturated_generation(&self, other: &Self) -> bool {
        self.allocation_credit != usize::MAX && self.allocation_credit == other.allocation_credit
    }

    pub(crate) fn inserted(&mut self) {
        self.allocation_credit = self.allocation_credit.saturating_add(1);
    }

    /// Merge another freshly deserialized native table's reservation into a
    /// conservative family bound. Earlier insertion credit is never rewound.
    pub(crate) fn include_tombstone_free_capacity(&mut self, capacity: usize) {
        self.allocation_credit = self.allocation_credit.max(capacity);
    }

    pub(crate) fn bucket_bound(&self) -> Option<usize> {
        // Pinned std/hashbrown may request twice the insertion credit before
        // applying its 7/8 load factor and power-of-two bucket rounding. Current
        // capacity() alone is not a retained bucket bound after deletion churn.
        if self.allocation_credit == 0 {
            return Some(0);
        }
        Some(
            (self.allocation_credit.checked_mul(2)?.checked_mul(8)? / 7)
                .checked_next_power_of_two()?
                .max(16),
        )
    }
}

#[derive(Clone)]
pub(crate) struct OwnedHashMap<K, V, S = RandomState> {
    inner: HashMap<K, V, S>,
    history: MapCloneHistory,
}

impl<K, V> OwnedHashMap<K, V> {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl<K, V, S: Default> Default for OwnedHashMap<K, V, S> {
    fn default() -> Self {
        Self {
            inner: HashMap::with_hasher(S::default()),
            history: MapCloneHistory::default(),
        }
    }
}

impl<K, V, S> OwnedHashMap<K, V, S> {
    pub(crate) fn clone_bucket_bound(&self) -> Option<usize> {
        self.history.bucket_bound()
    }

    pub(crate) fn clear(&mut self) {
        self.inner.clear();
    }

    pub(crate) fn values_mut(&mut self) -> hash_map::ValuesMut<'_, K, V> {
        self.inner.values_mut()
    }

    pub(crate) fn retain(&mut self, predicate: impl FnMut(&K, &mut V) -> bool) {
        self.inner.retain(predicate);
    }

    pub(crate) fn into_values(self) -> hash_map::IntoValues<K, V> {
        self.inner.into_values()
    }
}

impl<K: Eq + Hash, V, S: BuildHasher> OwnedHashMap<K, V, S> {
    pub(crate) fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Replacement overcounts by one; no extra hash/lookup is performed.
        self.history.inserted();
        self.inner.insert(key, value)
    }

    pub(crate) fn entry(&mut self, key: K) -> hash_map::Entry<'_, K, V> {
        let entry = self.inner.entry(key);
        if matches!(&entry, hash_map::Entry::Vacant(_)) {
            self.history.inserted();
        }
        entry
    }

    pub(crate) fn get_mut<Q: Eq + Hash + ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
    {
        self.inner.get_mut(key)
    }

    pub(crate) fn remove<Q: Eq + Hash + ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
    {
        self.inner.remove(key)
    }
}

// No DerefMut or raw mutable map escape: a borrow could grow then shrink the
// table without leaving enough evidence in its final len/capacity.
impl<K, V, S> Deref for OwnedHashMap<K, V, S> {
    type Target = HashMap<K, V, S>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for OwnedHashMap<K, V, S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl<K: Eq + Hash, V: PartialEq, S: BuildHasher> PartialEq for OwnedHashMap<K, V, S> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}
impl<K: Eq + Hash, V: Eq, S: BuildHasher> Eq for OwnedHashMap<K, V, S> {}

impl<K: Serialize, V: Serialize, S> Serialize for OwnedHashMap<K, V, S> {
    fn serialize<T: Serializer>(&self, serializer: T) -> Result<T::Ok, T::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, K: Deserialize<'de> + Eq + Hash, V: Deserialize<'de>, S: BuildHasher + Default>
    Deserialize<'de> for OwnedHashMap<K, V, S>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = HashMap::<K, V, S>::deserialize(deserializer)?;
        // serde_core 1.0.228 constructs this map with a cautious size-hint
        // reservation and insert only. This fresh result has no tombstones;
        // capacity supplies initial credit, including unused reservation. Never
        // use this rule for an arbitrary previously mutated map import.
        let history = MapCloneHistory::from_tombstone_free_capacity(inner.capacity());
        Ok(Self { inner, history })
    }
}

impl<'a, K, V, S> IntoIterator for &'a OwnedHashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = hash_map::Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut OwnedHashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = hash_map::IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_history_survives_churn_clear_and_native_clone_without_changing_map_semantics() {
        let mut map = OwnedHashMap::new();
        for key in 0..256 {
            map.insert(key, key * 2);
        }
        for key in 0..240 {
            map.remove(&key);
        }
        let bound = map.clone_bucket_bound().unwrap();
        let clone = map.clone();
        assert!(bound >= map.capacity());
        assert_eq!(clone.clone_bucket_bound(), Some(bound));
        assert_eq!(clone.capacity(), map.capacity());
        assert_eq!(
            clone.iter().collect::<Vec<_>>(),
            map.iter().collect::<Vec<_>>()
        );
        assert_eq!(format!("{map:?}"), format!("{:?}", map.inner));
        assert_eq!(
            serde_json::to_string(&map).unwrap(),
            serde_json::to_string(&map.inner).unwrap()
        );
        map.clear();
        assert_eq!(map.clone_bucket_bound(), Some(bound));
        assert_eq!(map, OwnedHashMap::new());
        // Default Clone::clone_from replaces storage, so source history never
        // describes a larger retained destination table.
        map.clone_from(&OwnedHashMap::new());
        assert_eq!(map.capacity(), 0);
        assert_eq!(map.clone_bucket_bound(), Some(0));
    }

    #[test]
    fn entry_keeps_native_lookup_and_factory_behavior_and_values_can_mutate() {
        use std::{cell::Cell, rc::Rc};
        #[derive(Clone)]
        struct Key(u32, Rc<Cell<usize>>);
        impl PartialEq for Key {
            fn eq(&self, other: &Self) -> bool {
                self.0 == other.0
            }
        }
        impl Eq for Key {}
        impl Hash for Key {
            fn hash<H: std::hash::Hasher>(&self, hasher: &mut H) {
                self.1.set(self.1.get() + 1);
                self.0.hash(hasher);
            }
        }
        let calls = Rc::new(Cell::new(0));
        let key = Key(7, calls.clone());
        let mut map = OwnedHashMap::new();
        let mut raw = HashMap::with_hasher(map.hasher().clone());
        let factories = Cell::new(0);
        *map.entry(key.clone()).or_insert_with(|| {
            factories.set(factories.get() + 1);
            1
        }) += 2;
        let tracked_hashes = calls.replace(0);
        *raw.entry(key.clone()).or_insert_with(|| {
            factories.set(factories.get() + 1);
            1
        }) += 2;
        assert_eq!(tracked_hashes, calls.replace(0));
        let credit = map.history.allocation_credit;
        map.entry(key.clone())
            .and_modify(|v| *v += 1)
            .or_insert_with(|| panic!("occupied"));
        let tracked_hashes = calls.replace(0);
        raw.entry(key.clone())
            .and_modify(|v| *v += 1)
            .or_insert_with(|| panic!("occupied"));
        assert_eq!(tracked_hashes, calls.replace(0));
        assert_eq!(factories.get(), 2);
        assert_eq!(map.history.allocation_credit, credit);
        assert_eq!(map.get(&key), raw.get(&key));
        *map.get_mut(&key).unwrap() += 1;
        for value in map.values_mut() {
            *value += 1;
        }
        for (_, value) in &mut map {
            *value += 1;
        }
        assert_eq!(map.get(&key), Some(&7));
        assert_eq!(map.history.allocation_credit, credit);
    }

    #[test]
    fn fresh_deserialize_covers_reservation_and_duplicates_without_exposing_history() {
        use serde::de::value::{Error, MapDeserializer};
        // An exact size hint reserves for many entries even though duplicate
        // keys collapse to one. Counting only the resulting len is unsound.
        let entries = vec![("key".to_owned(), 9u32); 256];
        let map = OwnedHashMap::<String, u32>::deserialize(MapDeserializer::<_, Error>::new(
            entries.into_iter(),
        ))
        .unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.capacity() >= 256);
        assert!(map.clone_bucket_bound().unwrap() >= map.capacity());
        let bytes = serde_json::to_string(&map).unwrap();
        assert_eq!(bytes, "{\"key\":9}");
        let restored: OwnedHashMap<String, u32> = serde_json::from_str(&bytes).unwrap();
        assert_eq!(restored, map);
        assert!(restored.clone_bucket_bound().unwrap() > 0);
    }

    #[test]
    fn overflow_preserves_ordinary_operations_and_cannot_be_mistaken_for_zero_cost() {
        let mut map = OwnedHashMap::new();
        map.history.allocation_credit = usize::MAX;
        assert_eq!(map.insert(1, 2), None);
        assert_eq!(map.remove(&1), Some(2));
        map.entry(2).or_insert(3);
        map.clear();
        assert_eq!(map.clone_bucket_bound(), None);
        assert_eq!(map.clone().clone_bucket_bound(), None);
        assert!(map.is_empty());
    }

    #[test]
    fn unused_vacant_entry_keeps_native_reservation_in_clone_history() {
        let mut map = OwnedHashMap::<u32, u32>::new();
        // Pinned std rustc_entry reserves before returning a Vacant entry.
        // Dropping it unused must not make that retained table cost disappear.
        {
            let _unused = map.entry(1);
        }
        assert!(map.is_empty());
        assert!(map.capacity() > 0);
        assert!(map.clone_bucket_bound().unwrap() >= map.capacity());
        map.clear();
        let clone = map.clone();
        assert_eq!(clone.capacity(), map.capacity());
        assert_eq!(clone.clone_bucket_bound(), map.clone_bucket_bound());
    }
}
