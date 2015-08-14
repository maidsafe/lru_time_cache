// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement, version 1.0.  This, along with the
// Licenses can be found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

#![crate_name = "lru_time_cache"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/dirvine/lru_time_cache/")]

#![forbid(bad_style, warnings)]

#![deny(deprecated, improper_ctypes, missing_docs, non_shorthand_field_patterns,
overflowing_literals, plugin_as_library, private_no_mangle_fns, private_no_mangle_statics,
raw_pointer_derive, stable_features, unconditional_recursion, unknown_lints, unsafe_code,
unused, unused_allocation, unused_attributes, unused_comparisons, unused_features,
unused_parens, while_true)]

#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
unused_qualifications, variant_size_differences)]

//!#lru cache limited via size or time
//!
//! This container allows time or size to be the limiting factor for any key/value types.
//!
//!#Use
//!
//!##To use as size based LruCache
//!
//!`let mut lru_cache = LruCache::<usize, usize>::with_capacity(size);`
//!
//!##Or as time based LruCache
//!
//! `let time_to_live = chrono::duration::Duration::milliseconds(100);`
//!
//! `let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(time_to_live);`
//!
//!##Or as time or size limited cache
//!
//! ` let size = 10usize;
//!     let time_to_live = chrono::duration::Duration::milliseconds(100);
//!     let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration_and_capacity(time_to_live, size);`

extern crate time;

use std::usize;
use std::collections::{BTreeMap, VecDeque};

/// A view into a single entry in a lru_cache, which may either be vacant or occupied.
pub enum Entry<'a, K:'a, V:'a> {
    /// A vacant Entry
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, V>),
}

/// A vacant Entry.
pub struct VacantEntry<'a, K:'a, V:'a> {
    key: K,
    cache: &'a mut LruCache<K, V>,
}

/// An occupied Entry.
pub struct OccupiedEntry<'a, V:'a> {
    value: &'a mut V,
}

/// Provides a Last Recently Used caching algorithm in a container which may be limited by size or time, reordered to most recently seen.
#[derive(Clone)]
pub struct LruCache<K, V> {
    map: BTreeMap<K, (V, time::SteadyTime)>,
    list: VecDeque<K>,
    capacity: usize,
    time_to_live: time::Duration,
}

/// Constructor for size (capacity) based LruCache
impl<K, V> LruCache<K, V> where K: PartialOrd + Ord + Clone, V: Clone {
    /// Constructor for size based LruCache
    pub fn with_capacity(capacity: usize) -> LruCache<K, V> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: capacity,
            time_to_live: time::Duration::max_value(),
        }
    }
    /// Constructor for time based LruCache
    pub fn with_expiry_duration(time_to_live: time::Duration) -> LruCache<K, V> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: usize::MAX,
            time_to_live: time_to_live,
        }
    }
    /// Constructor for dual feature capacity, or time based LruCache
    pub fn with_expiry_duration_and_capacity(time_to_live: time::Duration, capacity: usize) -> LruCache<K, V> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: capacity,
            time_to_live: time_to_live,
        }
    }

    /// Add a key/value pair to cache
    // FIXME: Should be deprecated in favor of the below `insert` function.
    pub fn add(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            while self.check_time_expired() || self.map.len() == self.capacity {
                self.remove_oldest_element();
            }

            self.list.push_back(key.clone());
            self.map.insert(key, (value, time::SteadyTime::now()));
        }
    }

    /// Inserts a key-value pair into the cache. If the key already had a value
    /// present in the cache, that value is returned. Otherwise, `None` is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.map.contains_key(&key) {
            Self::update_key(&mut self.list, &key);
        } else {
            while self.check_time_expired() || self.map.len() == self.capacity {
                self.remove_oldest_element();
            }
            self.list.push_back(key.clone());
        }

        self.map.insert(key, (value, time::SteadyTime::now())).map(|pair| pair.0)
    }

    /// Remove a key/value pair from cache
    pub fn remove(&mut self, key: &K)  -> Option<V> {
        let result = self.map.remove(key);

        if result.is_some() {
           let position = self.list.iter().enumerate().find(|a| !(*a.1 < *key || *a.1 > *key)).unwrap().0;
           self.list.remove(position);
           Some(result.unwrap().0)
        } else {
           None
        }
    }
    /// Retrieve a value from cache
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let list = &mut self.list;

        self.map.get(key).map(|result| {
            Self::update_key(list, key);
            &result.0
        })
    }
    /// Retrieve a value from cache
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let list = &mut self.list;

        self.map.get_mut(key).map(|result| {
            Self::update_key(list, key);
            &mut result.0
        })
    }

    /// Check for existence of a key
    // FIXME: Deprecated in favor of the `contains_key` function defined below.
    pub fn check(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Returns true if a value existed for the specified key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Current size of cache
    pub fn len(&self) -> usize {
        self.map.len()
    }

    // FIXME: We should really just implement the `iter` function for this Cache object,
    // let the user to clone and collect the elements when needed.
    /// Retrieve all elements as a vector of key value tuple.
    pub fn retrieve_all(&self) -> Vec<(K, V)> {
        let mut result = Vec::<(K, V)>::with_capacity(self.map.len());
        self.map.iter().all(|a| {
            result.push((a.0.clone(), a.1 .0.clone()));
            true
        });
        result
    }

    /// Return a vector of key value pairs ordered by most to least recently updated.
    pub fn retrieve_all_ordered(&self) -> Vec<(K, V)> {
        let mut result = Vec::<(K, V)>::with_capacity(self.list.len());
        for key in self.list.iter().rev() {
            match self.map.get(key) {
                Some(value) => result.push((key.clone(), value.0.clone())),
                None => (),
            }
        }
        result
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        // We need to do it the ugly way below due to this issue:
        // https://github.com/rust-lang/rfcs/issues/811
        //match self.get_mut(&key) {
        //    Some(value) => Entry::Occupied(OccupiedEntry{value: value}),
        //    None => Entry::Vacant(VacantEntry{key: key, cache: self}),
        //}
        if self.check(&key) {
            Entry::Occupied(OccupiedEntry{value: self.get_mut(&key).unwrap()})
        }
        else {
            Entry::Vacant(VacantEntry{key: key, cache: self})
        }
    }

    fn remove_oldest_element(&mut self) {
        self.list.pop_front().map(|key| { assert!(self.map.remove(&key).is_some()) });
    }

    fn check_time_expired(&self) -> bool {
        if self.time_to_live == time::Duration::max_value() || self.map.len() == 0 {
            false
        } else {
            self.map.get(self.list.front().unwrap()).unwrap().1 + self.time_to_live < time::SteadyTime::now()
        }
    }

    fn update_key(list: &mut VecDeque<K>, key: &K) {
        let pos_in_list = list.iter().enumerate().find(|a| !(*a.1 < *key || *a.1 > *key)).unwrap().0;
        list.remove(pos_in_list);
        list.push_back(key.clone());
    }
}

impl<'a, K: PartialOrd + Ord + Clone, V: Clone> VacantEntry<'a, K, V> {
    /// Inserts a value
    pub fn insert(self, value: V) -> &'a mut V {
        self.cache.insert(self.key.clone(), value);
        self.cache.get_mut(&self.key).unwrap()
    }
}

impl<'a, V: Clone> OccupiedEntry<'a, V> {
    /// Converts the entry into a mutable reference to its value.
    pub fn into_mut(self) -> &'a mut V {
        self.value
    }
}

impl<'a, K: PartialOrd + Ord + Clone, V: Clone> Entry<'a, K, V> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }
}

#[cfg(test)]
mod test {
    use time;
    extern crate rand;
    use std::thread;
    use super::LruCache;

    fn generate_random_vec<T>(len: usize) -> Vec<T> where T: rand::Rand {
        let mut vec = Vec::<T>::with_capacity(len);
        for _ in 0..len {
            vec.push(rand::random::<T>());
        }
        vec
    }

    #[test]
    fn size_only() {
        let size = 10usize;
        let mut lru_cache = LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            lru_cache.add(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        for i in 10..1000 {
            lru_cache.add(i, i);
            assert_eq!(lru_cache.len(), size);
        }

        for _ in (0..1000).rev() {
            assert!(lru_cache.check(&(1000 - 1)));
            assert!(lru_cache.get(&(1000 - 1)).is_some());
            assert_eq!(*lru_cache.get(&(1000 - 1)).unwrap(), 1000 - 1);
        }
    }

    #[test]
    fn time_only() {
        let time_to_live = time::Duration::milliseconds(100);
        let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            lru_cache.add(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        thread::sleep_ms(100);
        lru_cache.add(11, 11);

        assert_eq!(lru_cache.len(), 1);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i + 1);
            lru_cache.add(i, i);
            assert_eq!(lru_cache.len(), i + 2);
        }
    }

    #[test]
    fn time_and_size() {
        let size = 10usize;
        let time_to_live = time::Duration::milliseconds(100);
        let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration_and_capacity(time_to_live, size);

        for i in 0..1000 {
            if i < size {
                assert_eq!(lru_cache.len(), i);
            }

            lru_cache.add(i, i);

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        thread::sleep_ms(100);
        lru_cache.add(1, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[test]
    fn time_size_struct_value() {
        let size = 100usize;
        let time_to_live = time::Duration::milliseconds(100);

        #[derive(PartialEq, PartialOrd, Ord, Clone, Eq)]
        struct Temp {
            id: Vec<u8>,
        }

        let mut lru_cache = LruCache::<Temp, usize>::with_expiry_duration_and_capacity(time_to_live, size);

        for i in 0..1000 {
            if i < size {
                assert_eq!(lru_cache.len(), i);
            }

            lru_cache.add(Temp { id: generate_random_vec::<u8>(64), }, i);

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        thread::sleep_ms(100);
        lru_cache.add(Temp { id: generate_random_vec::<u8>(64), }, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[test]
    fn retrieve_all() {
        let size = 10usize;
        let mut lru_cache = LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            lru_cache.add(i, i);
        }

        let all = lru_cache.retrieve_all();
        assert_eq!(all.len(), lru_cache.map.len());

        assert!(all.iter().all(|a| lru_cache.check(&a.0) && *lru_cache.get(&a.0).unwrap() == a.1));
    }

    #[test]
    fn retrieve_all_ordered() {
        let size = 10usize;
        let mut lru_cache = LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            lru_cache.add(i, i);
        }

        let all = lru_cache.retrieve_all_ordered();
        assert_eq!(all.len(), lru_cache.map.len());

        for i in all.iter().rev() {
            lru_cache.remove_oldest_element();
            assert!(!lru_cache.check(&i.0) && lru_cache.get(&i.0).is_none());
        }
    }
}
