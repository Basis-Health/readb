use crate::cache::{Cache, Key, Value};
use std::cmp::Reverse;
use std::collections::{hash_map::Entry, BinaryHeap, HashMap};

const DEFAULT_CAPACITY: usize = 1024;

pub struct LfuCache {
    capacity: usize,
    map: HashMap<Key, (Value, usize)>, // key to (value, frequency)
    frequencies: BinaryHeap<Reverse<(usize, Key)>>, // Reverse heap of (frequency, key)
}

impl LfuCache {
    pub fn new(capacity: usize) -> LfuCache {
        LfuCache {
            capacity,
            map: HashMap::new(),
            frequencies: BinaryHeap::new(),
        }
    }
}

impl Cache for LfuCache {
    fn new(size: usize) -> Self {
        Self::new(size)
    }

    fn new_default() -> Self {
        Self::new(DEFAULT_CAPACITY)
    }

    fn get(&mut self, key: &Key) -> Option<Value> {
        match self.map.get_mut(key) {
            Some((value, frequency)) => {
                *frequency += 1;
                self.frequencies.push(Reverse((*frequency, *key)));
                Some(value.clone())
            }
            None => None,
        }
    }

    fn put(&mut self, key: Key, value: Value) {
        if self.map.len() == self.capacity {
            while let Some(Reverse((_, evicted_key))) = self.frequencies.pop() {
                if let Entry::Occupied(e) = self.map.entry(evicted_key) {
                    if e.get().1 > 1 {
                        continue;
                    }
                    e.remove_entry();
                    break;
                }
            }
        }

        self.map.insert(key, (value, 1));
        self.frequencies.push(Reverse((1, key)));
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::lfu::LfuCache;
    use crate::cache::Cache;

    #[test]
    fn create_lfu_cache() {
        let cache = LfuCache::new(2);
        assert_eq!(cache.capacity, 2);
        assert_eq!(cache.map.len(), 0);
    }

    #[test]
    fn put_get_elements() {
        let mut cache = LfuCache::new(2);
        cache.put((1, 0), "a".to_string().as_bytes().to_vec());
        cache.put((2, 0), "b".to_string().as_bytes().to_vec());

        assert_eq!(
            cache.get(&(1, 0)),
            Some("a".to_string().as_bytes().to_vec())
        );
        assert_eq!(
            cache.get(&(2, 0)),
            Some("b".to_string().as_bytes().to_vec())
        );
    }

    #[test]
    fn evict_least_frequent() {
        let mut cache = LfuCache::new(2);
        cache.put((1, 0), "a".to_string().as_bytes().to_vec());
        cache.put((2, 0), "b".to_string().as_bytes().to_vec());
        cache.get(&(1, 0));
        cache.put((3, 0), "c".to_string().as_bytes().to_vec());

        assert_eq!(
            cache.get(&(1, 0)),
            Some("a".to_string().as_bytes().to_vec())
        );
        assert_eq!(cache.get(&(2, 0)), None);
        assert_eq!(
            cache.get(&(3, 0)),
            Some("c".to_string().as_bytes().to_vec())
        );
    }

    #[test]
    fn increment_frequency() {
        let mut cache = LfuCache::new(2);
        cache.put((1, 0), "a".to_string().as_bytes().to_vec());
        cache.get(&(1, 0));
        cache.get(&(1, 0));

        assert_eq!(cache.map.get(&(1, 0)).unwrap().1, 3);
    }
}
