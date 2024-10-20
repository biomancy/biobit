use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

pub trait BundleOp {
    type Key;
    type Value;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value>;
}

impl<K: Hash + Eq, V, S: std::hash::BuildHasher> BundleOp for HashMap<K, V, S> {
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        HashMap::get(&self, key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        HashMap::get_mut(self, key)
    }
}

impl<K: Ord, V> BundleOp for BTreeMap<K, V> {
    type Key = K;
    type Value = V;

    fn get(&self, key: &Self::Key) -> Option<&Self::Value> {
        BTreeMap::get(&self, key)
    }

    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Value> {
        BTreeMap::get_mut(self, key)
    }
}
