// Copyright 2015 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under (1) the MaidSafe.net Commercial License,
// version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
// licence you accepted on initial access to the Software (the "Licences").
//
// By contributing code to the SAFE Network Software, or to this project generally, you agree to be
// bound by the terms of the MaidSafe Contributor Agreement.  This, along with the Licenses can be
// found in the root directory of this project at LICENSE, COPYING and CONTRIBUTOR.
//
// Unless required by applicable law or agreed to in writing, the SAFE Network Software distributed
// under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.
//
// Please review the Licences for the specific language governing permissions and limitations
// relating to use of the SAFE Network Software.

//! # Least Recently Used (LRU) Cache
//!
//! Implementation of a Least Recently Used
//! [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be
//! limited by size or time, ordered by most recently seen.
//!
//! # Examples
//!
//! ```
//! extern crate lru_time_cache;
//! use lru_time_cache::LruCache;
//!
//! # fn main() {
//! // Construct an `LruCache` of `<u8, String>`s, limited by key count
//! let max_count = 10;
//! let _lru_cache = LruCache::<u8, String>::with_capacity(max_count);
//!
//! // Construct an `LruCache` of `<String, i64>`s, limited by expiry time
//! let time_to_live = ::std::time::Duration::from_millis(100);
//! let _lru_cache = LruCache::<String, i64>::with_expiry_duration(time_to_live);
//!
//! // Construct an `LruCache` of `<u64, Vec<u8>>`s, limited by key count and expiry time
//! let _lru_cache = LruCache::<u64, Vec<u8>>::with_expiry_duration_and_capacity(time_to_live,
//!                                                                              max_count);
//! # }
//! ```

#![doc(html_logo_url =
           "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "https://maidsafe.net/img/favicon.ico",
       html_root_url = "https://docs.rs/lru_time_cache")]

// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(bad_style, exceeding_bitshifts, mutable_transmutes, no_mangle_const_items,
          unknown_crate_types, warnings)]
#![deny(deprecated, improper_ctypes, missing_docs,
        non_shorthand_field_patterns, overflowing_literals, plugin_as_library,
        private_no_mangle_fns, private_no_mangle_statics, stable_features, unconditional_recursion,
        unknown_lints, unsafe_code, unused, unused_allocation, unused_attributes,
        unused_comparisons, unused_features, unused_parens, while_true)]
#![warn(trivial_casts, trivial_numeric_casts, unused_extern_crates, unused_import_braces,
        unused_qualifications, unused_results)]
#![allow(box_pointers, fat_ptr_transmutes, missing_copy_implementations,
         missing_debug_implementations, variant_size_differences)]

#[cfg(test)]
extern crate rand;
#[cfg(feature = "fake_clock")]
extern crate fake_clock;

#[cfg(feature = "fake_clock")]
use fake_clock::FakeClock as Instant;
use std::borrow::Borrow;
use std::collections::{BTreeMap, VecDeque, btree_map};
use std::time::Duration;
#[cfg(not(feature = "fake_clock"))]
use std::time::Instant;
use std::usize;

/// A view into a single entry in an LRU cache, which may either be vacant or occupied.
pub enum Entry<'a, Key: 'a, Value: 'a> {
    /// A vacant Entry
    Vacant(VacantEntry<'a, Key, Value>),
    /// An occupied Entry
    Occupied(OccupiedEntry<'a, Value>),
}

/// A vacant Entry.
pub struct VacantEntry<'a, Key: 'a, Value: 'a> {
    key: Key,
    cache: &'a mut LruCache<Key, Value>,
}

/// An occupied Entry.
pub struct OccupiedEntry<'a, Value: 'a> {
    value: &'a mut Value,
}

/// An iterator over an `LruCache`'s entries that updates the timestamps as values are traversed.
pub struct Iter<'a, Key: 'a, Value: 'a> {
    map_iter_mut: btree_map::IterMut<'a, Key, (Value, Instant)>,
    list: &'a mut VecDeque<Key>,
    has_expiry: bool,
    lru_cache_ttl: Duration,
}

impl<'a, Key, Value> Iterator for Iter<'a, Key, Value>
where
    Key: Ord + Clone,
{
    type Item = (&'a Key, &'a Value);

    #[cfg_attr(feature = "cargo-clippy", allow(while_let_on_iterator))]
    fn next(&mut self) -> Option<(&'a Key, &'a Value)> {
        let now = Instant::now();
        while let Some((key, &mut (ref value, ref mut instant))) = self.map_iter_mut.next() {
            if !self.has_expiry || *instant + self.lru_cache_ttl > now {
                LruCache::<Key, Value>::update_key(self.list, key);
                *instant = now;
                return Some((key, value));
            }
        }
        None
    }
}

/// An iterator over an `LruCache`'s entries that does not modify the timestamp.
pub struct PeekIter<'a, Key: 'a, Value: 'a> {
    map_iter: btree_map::Iter<'a, Key, (Value, Instant)>,
    lru_cache: &'a LruCache<Key, Value>,
}

impl<'a, Key, Value> Iterator for PeekIter<'a, Key, Value>
where
    Key: Ord + Clone,
{
    type Item = (&'a Key, &'a Value);

    #[cfg_attr(feature = "cargo-clippy", allow(while_let_on_iterator))]
    fn next(&mut self) -> Option<(&'a Key, &'a Value)> {
        while let Some((key, &(ref value, _))) = self.map_iter.next() {
            if !self.lru_cache.expired(key) {
                return Some((key, value));
            }
        }
        None
    }
}

/// Implementation of [LRU cache](index.html#least-recently-used-lru-cache).
pub struct LruCache<Key, Value> {
    map: BTreeMap<Key, (Value, Instant)>,
    list: VecDeque<Key>,
    capacity: usize,
    time_to_live: Duration,
}

impl<Key, Value> LruCache<Key, Value>
where
    Key: Ord + Clone,
{
    /// Constructor for capacity based `LruCache`.
    pub fn with_capacity(capacity: usize) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: capacity,
            time_to_live: Duration::new(std::u64::MAX, 999_999_999),
        }
    }

    /// Constructor for time based `LruCache`.
    pub fn with_expiry_duration(time_to_live: Duration) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: usize::MAX,
            time_to_live: time_to_live,
        }
    }

    /// Constructor for dual-feature capacity and time based `LruCache`.
    pub fn with_expiry_duration_and_capacity(
        time_to_live: Duration,
        capacity: usize,
    ) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: capacity,
            time_to_live: time_to_live,
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the key already existed in the cache, the existing value is returned and overwritten in
    /// the cache.  Otherwise, the key-value pair is inserted and `None` is returned.
    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        if self.map.contains_key(&key) {
            Self::update_key(&mut self.list, &key);
        } else {
            while self.check_time_expired() || self.map.len() == self.capacity {
                self.remove_oldest_element();
            }
            self.list.push_back(key.clone());
        }

        self.map.insert(key, (value, Instant::now())).map(
            |pair| pair.0,
        )
    }

    /// Removes a key-value pair from the cache.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.list.retain(
            |k| *k.borrow() < *key || *k.borrow() > *key,
        );
        self.map.remove(key).map(|(value, _)| value)
    }

    /// Clears the `LruCache`, removing all values.
    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();
    }

    /// Retrieves a reference to the value stored under `key`, or `None` if the key doesn't exist.
    /// Also removes expired elements and updates the time.
    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.remove_expired();
        let list = &mut self.list;

        self.map.get_mut(key).map(|result| {
            Self::update_key(list, key);
            result.1 = Instant::now();
            &result.0
        })
    }

    /// Returns a reference to the value with the given `key`, if present and not expired, without
    /// updating the timestamp.
    pub fn peek<Q: ?Sized>(&self, key: &Q) -> Option<&Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        if self.expired(key) {
            return None;
        }
        self.map.get(key).map(|&(ref value, _)| value)
    }

    /// Retrieves a mutable reference to the value stored under `key`, or `None` if the key doesn't
    /// exist.  Also removes expired elements and updates the time.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.remove_expired();
        let list = &mut self.list;

        self.map.get_mut(key).map(|result| {
            Self::update_key(list, key);
            result.1 = Instant::now();
            &mut result.0
        })
    }

    /// Returns whether `key` exists in the cache or not.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.map.contains_key(key) && !self.expired(key)
    }

    /// Returns the size of the cache, i.e. the number of cached non-expired key-value pairs.
    pub fn len(&self) -> usize {
        self.map.len() - self.list.iter().take_while(|key| self.expired(key)).count()
    }

    /// Returns `true` if there are no non-expired entries in the cache.
    pub fn is_empty(&self) -> bool {
        self.list.iter().all(|key| self.expired(key))
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: Key) -> Entry<Key, Value> {
        // We need to do it the ugly way below due to this issue:
        // https://github.com/rust-lang/rfcs/issues/811
        // match self.get_mut(&key) {
        //     Some(value) => Entry::Occupied(OccupiedEntry{value: value}),
        //     None => Entry::Vacant(VacantEntry{key: key, cache: self}),
        // }
        if self.contains_key(&key) {
            Entry::Occupied(OccupiedEntry {
                value: self.get_mut(&key).expect("key not found"),
            })
        } else {
            Entry::Vacant(VacantEntry {
                key: key,
                cache: self,
            })
        }
    }

    /// Returns an iterator over all entries that updates the timestamps as values are
    /// traversed. Also removes expired elements before creating the iterator.
    pub fn iter(&mut self) -> Iter<Key, Value> {
        self.remove_expired();

        let has_expiry = self.has_expiry();

        Iter {
            map_iter_mut: self.map.iter_mut(),
            list: &mut self.list,
            has_expiry: has_expiry,
            lru_cache_ttl: self.time_to_live,
        }
    }

    /// Returns an iterator over all entries that does not modify the timestamps.
    pub fn peek_iter(&self) -> PeekIter<Key, Value> {
        PeekIter {
            map_iter: self.map.iter(),
            lru_cache: self,
        }
    }

    fn has_expiry(&self) -> bool {
        self.time_to_live != Duration::new(std::u64::MAX, 999_999_999)
    }

    fn expired<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        let now = Instant::now();
        self.has_expiry() &&
            self.map.get(key).map_or(
                false,
                |v| v.1 + self.time_to_live < now,
            )
    }

    fn remove_oldest_element(&mut self) {
        let _ = self.list.pop_front().map(|key| {
            assert!(self.map.remove(&key).is_some())
        });
    }

    fn check_time_expired(&self) -> bool {
        self.has_expiry() && self.list.front().map_or(false, |key| self.expired(key))
    }

    // Move `key` in the ordered list to the last
    fn update_key<Q: ?Sized>(list: &mut VecDeque<Key>, key: &Q)
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        if let Some(pos) = list.iter().position(|k| k.borrow() == key) {
            let k = list.remove(pos).unwrap();
            list.push_back(k);
        }
    }

    fn remove_expired(&mut self) {
        while self.check_time_expired() {
            self.remove_oldest_element();
        }
    }
}

impl<Key, Value> Clone for LruCache<Key, Value>
where
    Key: Clone,
    Value: Clone,
{
    fn clone(&self) -> LruCache<Key, Value> {
        LruCache {
            map: self.map.clone(),
            list: self.list.clone(),
            capacity: self.capacity,
            time_to_live: self.time_to_live,
        }
    }
}

impl<'a, Key: Ord + Clone, Value> VacantEntry<'a, Key, Value> {
    /// Inserts a value
    pub fn insert(self, value: Value) -> &'a mut Value {
        let _ = self.cache.insert(self.key.clone(), value);
        self.cache.get_mut(&self.key).expect("key not found")
    }
}

impl<'a, Value> OccupiedEntry<'a, Value> {
    /// Converts the entry into a mutable reference to its value.
    pub fn into_mut(self) -> &'a mut Value {
        self.value
    }
}

impl<'a, Key: Ord + Clone, Value> Entry<'a, Key, Value> {
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: Value) -> &'a mut Value {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> Value>(self, default: F) -> &'a mut Value {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }
}

#[cfg(test)]
mod test {
    use rand;
    use std::time::Duration;

    #[cfg(feature = "fake_clock")]
    fn sleep(time: u64) {
        use fake_clock::FakeClock;
        FakeClock::advance_time(time);
    }

    #[cfg(not(feature = "fake_clock"))]
    fn sleep(time: u64) {
        use std::thread;
        thread::sleep(Duration::from_millis(time));
    }

    fn generate_random_vec<T>(len: usize) -> Vec<T>
    where
        T: rand::Rand,
    {
        let mut vec = Vec::<T>::with_capacity(len);
        for _ in 0..len {
            vec.push(rand::random());
        }
        vec
    }

    #[test]
    fn size_only() {
        let size = 10usize;
        let mut lru_cache = super::LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        for i in 10..1000 {
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), size);
        }

        for _ in (0..1000).rev() {
            assert!(lru_cache.contains_key(&(1000 - 1)));
            assert!(lru_cache.get(&(1000 - 1)).is_some());
            assert_eq!(*lru_cache.get(&(1000 - 1)).unwrap(), 1000 - 1);
        }
    }

    #[test]
    fn time_only() {
        let time_to_live = Duration::from_millis(100);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        sleep(101);
        let _ = lru_cache.insert(11, 11);

        assert_eq!(lru_cache.len(), 1);

        for i in 0..10 {
            assert!(!lru_cache.is_empty());
            assert_eq!(lru_cache.len(), i + 1);
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), i + 2);
        }

        sleep(101);
        assert_eq!(0, lru_cache.len());
        assert!(lru_cache.is_empty());
    }

    #[test]
    fn time_only_check() {
        let time_to_live = Duration::from_millis(50);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        assert_eq!(lru_cache.len(), 0);
        let _ = lru_cache.insert(0, 0);
        assert_eq!(lru_cache.len(), 1);

        sleep(101);

        assert!(!lru_cache.contains_key(&0));
        assert_eq!(lru_cache.len(), 0);
    }

    #[test]
    fn time_and_size() {
        let size = 10usize;
        let time_to_live = Duration::from_millis(100);
        let mut lru_cache =
            super::LruCache::<usize, usize>::with_expiry_duration_and_capacity(time_to_live, size);

        for i in 0..1000 {
            if i < size {
                assert_eq!(lru_cache.len(), i);
            }

            let _ = lru_cache.insert(i, i);

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        sleep(101);
        let _ = lru_cache.insert(1, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[derive(PartialEq, PartialOrd, Ord, Clone, Eq)]
    struct Temp {
        id: Vec<u8>,
    }

    #[test]
    fn time_size_struct_value() {
        let size = 100usize;
        let time_to_live = Duration::from_millis(100);

        let mut lru_cache =
            super::LruCache::<Temp, usize>::with_expiry_duration_and_capacity(time_to_live, size);

        for i in 0..1000 {
            if i < size {
                assert_eq!(lru_cache.len(), i);
            }

            let _ = lru_cache.insert(Temp { id: generate_random_vec::<u8>(64) }, i);

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        sleep(101);
        let _ = lru_cache.insert(Temp { id: generate_random_vec::<u8>(64) }, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[test]
    fn iter() {
        let mut lru_cache = super::LruCache::<usize, usize>::with_capacity(3);

        let _ = lru_cache.insert(0, 0);
        sleep(1);
        let _ = lru_cache.insert(1, 1);
        sleep(1);
        let _ = lru_cache.insert(2, 2);
        sleep(1);

        assert_eq!(
            vec![(&0, &0), (&1, &1), (&2, &2)],
            lru_cache.iter().collect::<Vec<_>>()
        );

        let initial_instant0 = lru_cache.map[&0].1;
        let initial_instant2 = lru_cache.map[&2].1;
        sleep(1);

        // only the first two entries should have their timestamp updated (and position in list)
        let _ = lru_cache.iter().take(2).all(|_| true);

        assert_ne!(lru_cache.map[&0].1, initial_instant0);
        assert_eq!(lru_cache.map[&2].1, initial_instant2);

        assert_eq!(*lru_cache.list.front().unwrap(), 2);
        assert_eq!(*lru_cache.list.back().unwrap(), 1);
    }

    #[test]
    fn peek_iter() {
        let time_to_live = Duration::from_millis(50);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        let _ = lru_cache.insert(0, 0);
        let _ = lru_cache.insert(2, 2);
        let _ = lru_cache.insert(3, 3);

        sleep(30);
        assert_eq!(
            vec![(&0, &0), (&2, &2), (&3, &3)],
            lru_cache.peek_iter().collect::<Vec<_>>()
        );
        assert_eq!(Some(&2), lru_cache.get(&2));
        let _ = lru_cache.insert(1, 1);
        let _ = lru_cache.insert(4, 4);

        sleep(30);
        assert_eq!(
            vec![(&1, &1), (&2, &2), (&4, &4)],
            lru_cache.peek_iter().collect::<Vec<_>>()
        );

        sleep(30);
        assert!(lru_cache.is_empty());
    }

    #[test]
    fn update_time_check() {
        let time_to_live = Duration::from_millis(50);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        assert_eq!(lru_cache.len(), 0);
        let _ = lru_cache.insert(0, 0);
        assert_eq!(lru_cache.len(), 1);

        sleep(30);
        assert_eq!(Some(&0), lru_cache.get(&0));
        sleep(30);
        assert_eq!(Some(&0), lru_cache.peek(&0));
        sleep(30);
        assert_eq!(None, lru_cache.peek(&0));
    }

    #[test]
    fn deref_coercions() {
        let mut lru_cache = super::LruCache::<String, usize>::with_capacity(1);
        let _ = lru_cache.insert("foo".to_string(), 0);
        assert_eq!(true, lru_cache.contains_key("foo"));
        assert_eq!(Some(&0), lru_cache.get("foo"));
        assert_eq!(Some(&mut 0), lru_cache.get_mut("foo"));
        assert_eq!(Some(&0), lru_cache.peek("foo"));
        assert_eq!(Some(0), lru_cache.remove("foo"));
    }
}
