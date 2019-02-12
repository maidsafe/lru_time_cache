// Copyright 2018 MaidSafe.net limited.
//
// This SAFE Network Software is licensed to you under the MIT license <LICENSE-MIT
// http://opensource.org/licenses/MIT> or the Modified BSD license <LICENSE-BSD
// https://opensource.org/licenses/BSD-3-Clause>, at your option. This file may not be copied,
// modified, or distributed except according to those terms. Please review the Licences for the
// specific language governing permissions and limitations relating to use of the SAFE Network
// Software.

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

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
    html_favicon_url = "https://maidsafe.net/img/favicon.ico",
    test(attr(forbid(warnings)))
)]
// For explanation of lint checks, run `rustc -W help` or see
// https://github.com/maidsafe/QA/blob/master/Documentation/Rust%20Lint%20Checks.md
#![forbid(
    bad_style,
    exceeding_bitshifts,
    mutable_transmutes,
    no_mangle_const_items,
    unknown_crate_types,
    warnings
)]
#![deny(
    deprecated,
    improper_ctypes,
    missing_docs,
    non_shorthand_field_patterns,
    overflowing_literals,
    plugin_as_library,
    stable_features,
    unconditional_recursion,
    unknown_lints,
    unsafe_code,
    unused,
    unused_allocation,
    unused_attributes,
    unused_comparisons,
    unused_features,
    unused_parens,
    while_true
)]
#![warn(
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![allow(
    box_pointers,
    missing_copy_implementations,
    missing_debug_implementations,
    variant_size_differences
)]

#[cfg(test)]
extern crate rand;

#[cfg(feature = "fake_clock")]
use fake_clock::FakeClock as Instant;
use std::borrow::Borrow;
use std::collections::{btree_map, BTreeMap, VecDeque};
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
/// Values are produced in the most recently used order.
pub struct Iter<'a, Key: 'a, Value: 'a> {
    /// Reference to the iterated cache.
    map: &'a mut BTreeMap<Key, (Value, Instant)>,
    /// Ordered cache entry keys where the least recently used items are first.
    list: &'a mut VecDeque<Key>,
    lru_cache_ttl: Option<Duration>,
    /// Index in `list` of the previously used item.
    item_index: usize,
}

impl<'a, Key, Value> Iter<'a, Key, Value>
where
    Key: Ord,
{
    /// Returns next unexpired item in the cache or `None` if no such items.
    /// Expired items are removed from the cache.
    fn next_unexpired(&mut self, now: Instant) -> Option<Key> {
        loop {
            self.item_index = self.item_index.checked_sub(1)?;
            let key = self.list.remove(self.item_index)?;
            let value = self.map.get(&key)?;

            if let Some(ttl) = self.lru_cache_ttl {
                if value.1 + ttl > now {
                    return Some(key);
                } else {
                    let _ = self.map.remove(&key);
                }
            } else {
                return Some(key);
            }
        }
    }
}

impl<'a, Key, Value> Iterator for Iter<'a, Key, Value>
where
    Key: Ord + Clone,
{
    type Item = (&'a Key, &'a Value);

    /// Returns the next element in the cache and moves it to the top of the cache.
    /// The most recently used items are yield first.
    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<(&'a Key, &'a Value)> {
        let now = Instant::now();
        let key = self.next_unexpired(now)?;
        self.list.push_back(key);
        let key = self.list.back()?;
        let mut value = self.map.get_mut(&key)?;
        value.1 = now;

        unsafe {
            let key = std::mem::transmute::<&Key, &'a Key>(key);
            let value = std::mem::transmute::<&Value, &'a Value>(&value.0);
            Some((key, value))
        }
    }
}

/// Entry produced by `NotifyIter` that might be still valid or expired.
pub enum TimedEntry<'a, Key: 'a, Value: 'a> {
    /// Entry has not yet expired.
    Valid(&'a Key, &'a Value),
    /// Entry got expired and was evicted from the cache.
    Expired(Key, Value),
}

/// Much like `Iter` except will produce expired entries too where `Iter` silently drops them.
pub struct NotifyIter<'a, Key: 'a, Value: 'a> {
    /// Reference to the iterated cache.
    map: &'a mut BTreeMap<Key, (Value, Instant)>,
    /// Ordered cache entry keys where the least recently used items are first.
    list: &'a mut VecDeque<Key>,
    lru_cache_ttl: Option<Duration>,
    /// Index in `list` of the previously used item.
    item_index: usize,
}

impl<'a, Key, Value> Iterator for NotifyIter<'a, Key, Value>
where
    Key: Ord + Clone,
{
    type Item = TimedEntry<'a, Key, Value>;

    /// Returns the next element in the cache and moves it to the top of the cache.
    /// The most recently used items are yield first.
    #[allow(unsafe_code)]
    fn next(&mut self) -> Option<Self::Item> {
        self.item_index = self.item_index.checked_sub(1)?;
        let key = self.list.remove(self.item_index)?;
        let mut value = self.map.get_mut(&key)?;
        let now = Instant::now();

        if let Some(ttl) = self.lru_cache_ttl {
            if value.1 + ttl <= now {
                let value = self.map.remove(&key)?;
                return Some(TimedEntry::Expired(key, value.0));
            }
        }

        self.list.push_back(key);
        let key = self.list.back()?;
        value.1 = now;
        unsafe {
            let key = std::mem::transmute::<&Key, &'a Key>(key);
            let value = std::mem::transmute::<&Value, &'a Value>(&value.0);
            Some(TimedEntry::Valid(key, value))
        }
    }
}

/// An iterator over an `LruCache`'s entries that does not modify the timestamp.
pub struct PeekIter<'a, Key: 'a, Value: 'a> {
    map_iter: btree_map::Iter<'a, Key, (Value, Instant)>,
    lru_cache_ttl: Option<Duration>,
}

impl<'a, Key, Value> Iterator for PeekIter<'a, Key, Value>
where
    Key: Ord + Clone,
{
    type Item = (&'a Key, &'a Value);

    fn next(&mut self) -> Option<(&'a Key, &'a Value)> {
        let now = Instant::now();
        let not_expired = match self.lru_cache_ttl {
            Some(ttl) => self
                .map_iter
                .find(|&(_, &(_, instant))| instant + ttl > now),
            None => self.map_iter.next(),
        };
        not_expired.map(|(key, &(ref value, _))| (key, value))
    }
}

/// Implementation of [LRU cache](index.html#least-recently-used-lru-cache).
pub struct LruCache<Key, Value> {
    map: BTreeMap<Key, (Value, Instant)>,
    list: VecDeque<Key>,
    capacity: usize,
    time_to_live: Option<Duration>,
}

impl<Key, Value> LruCache<Key, Value>
where
    Key: Ord + Clone,
{
    /// Constructor for capacity based `LruCache`.
    pub fn with_capacity(capacity: usize) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::with_capacity(capacity),
            capacity,
            time_to_live: None,
        }
    }

    /// Constructor for time based `LruCache`.
    pub fn with_expiry_duration(time_to_live: Duration) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::new(),
            capacity: usize::MAX,
            time_to_live: Some(time_to_live),
        }
    }

    /// Constructor for dual-feature capacity and time based `LruCache`.
    pub fn with_expiry_duration_and_capacity(
        time_to_live: Duration,
        capacity: usize,
    ) -> LruCache<Key, Value> {
        LruCache {
            map: BTreeMap::new(),
            list: VecDeque::with_capacity(capacity),
            capacity,
            time_to_live: Some(time_to_live),
        }
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the key already existed in the cache, the existing value is returned and overwritten in
    /// the cache.  Otherwise, the key-value pair is inserted and `None` is returned.
    /// Evicts and returns expired entries.
    pub fn notify_insert(&mut self, key: Key, value: Value) -> (Option<Value>, Vec<(Key, Value)>) {
        let expired = self.remove_expired();

        if self.map.contains_key(&key) {
            Self::update_key(&mut self.list, &key);
        } else {
            self.remove_lru();
            self.list.push_back(key.clone());
        };

        (
            self.map
                .insert(key, (value, Instant::now()))
                .map(|pair| pair.0),
            expired,
        )
    }

    /// Inserts a key-value pair into the cache.
    ///
    /// If the key already existed in the cache, the existing value is returned and overwritten in
    /// the cache.  Otherwise, the key-value pair is inserted and `None` is returned.
    pub fn insert(&mut self, key: Key, value: Value) -> Option<Value> {
        self.notify_insert(key, value).0
    }

    /// Removes a key-value pair from the cache.
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.map.remove(key).map(|(value, _)| {
            let _ = self
                .list
                .iter()
                .position(|l| l.borrow() == key)
                .map(|p| self.list.remove(p));
            value
        })
    }

    /// Clears the `LruCache`, removing all values.
    pub fn clear(&mut self) {
        self.map.clear();
        self.list.clear();
    }

    /// Much like `get()`, except in addition returns expired entries.
    pub fn notify_get<Q: ?Sized>(&mut self, key: &Q) -> (Option<&Value>, Vec<(Key, Value)>)
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        let (value, expired) = self.notify_get_mut(key);
        (value.map(|v| &*v), expired)
    }

    /// Retrieves a reference to the value stored under `key`, or `None` if the key doesn't exist.
    /// Also removes expired elements and updates the time.
    pub fn get<Q: ?Sized>(&mut self, key: &Q) -> Option<&Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.get_mut(key).map(|v| &*v)
    }

    /// Returns a reference to the value with the given `key`, if present and not expired, without
    /// updating the timestamp.
    pub fn peek<Q: ?Sized>(&self, key: &Q) -> Option<&Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.map
            .get(key)
            .into_iter()
            .find(|&(_, t)| {
                self.time_to_live
                    .map_or(true, |ttl| *t + ttl >= Instant::now())
            })
            .map(|&(ref value, _)| value)
    }

    /// Retrieves a mutable reference to the value stored under `key`, or `None` if the key doesn't
    /// exist. Also removes expired elements and updates the time.
    pub fn notify_get_mut<Q: ?Sized>(&mut self, key: &Q) -> (Option<&mut Value>, Vec<(Key, Value)>)
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        let expired = self.remove_expired();

        let list = &mut self.list;
        (
            self.map.get_mut(key).map(|result| {
                Self::update_key(list, key);
                result.1 = Instant::now();
                &mut result.0
            }),
            expired,
        )
    }

    /// Retrieves a mutable reference to the value stored under `key`, or `None` if the key doesn't
    /// exist.  Also removes expired elements and updates the time.
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut Value>
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.notify_get_mut(key).0
    }

    /// Returns whether `key` exists in the cache or not.
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        self.peek(key).is_some()
    }

    /// Returns the size of the cache, i.e. the number of cached non-expired key-value pairs.
    pub fn len(&self) -> usize {
        // FIXME: we assume most items are not expired => it is faster to count the expired ones.
        //
        // If this assumption is not valid, then directly iterating through all the
        // map items and counting the not expired ones would be faster (no map lookups)
        self.time_to_live.map_or(self.list.len(), |ttl| {
            self.list
                .iter()
                .filter_map(|key| self.map.get(key))
                .position(|&(_, t)| t + ttl >= Instant::now())
                .map_or(0, |p| self.map.len() - p)
        })
    }

    /// Returns `true` if there are no non-expired entries in the cache.
    pub fn is_empty(&self) -> bool {
        self.time_to_live.map_or(self.list.is_empty(), |ttl| {
            self.list
                .back()
                .and_then(|key| self.map.get(key))
                .map_or(true, |&(_, t)| t + ttl < Instant::now())
        })
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
            Entry::Vacant(VacantEntry { key, cache: self })
        }
    }

    /// Returns an iterator over all entries that updates the timestamps as values are
    /// traversed. Also removes expired elements before creating the iterator.
    /// Values are produced in the most recently used order.
    ///
    /// Also, evicts and returns expired entries.
    pub fn notify_iter(&mut self) -> NotifyIter<Key, Value> {
        NotifyIter {
            item_index: self.list.len(),
            map: &mut self.map,
            list: &mut self.list,
            lru_cache_ttl: self.time_to_live,
        }
    }

    /// Returns an iterator over all entries that updates the timestamps as values are
    /// traversed. Also removes expired elements before creating the iterator.
    /// Values are produced in the most recently used order.
    pub fn iter(&mut self) -> Iter<Key, Value> {
        let _ = self.remove_expired();

        Iter {
            item_index: self.list.len(),
            map: &mut self.map,
            list: &mut self.list,
            lru_cache_ttl: self.time_to_live,
        }
    }

    /// Returns an iterator over all entries that does not modify the timestamps.
    pub fn peek_iter(&self) -> PeekIter<Key, Value> {
        PeekIter {
            map_iter: self.map.iter(),
            lru_cache_ttl: self.time_to_live,
        }
    }

    // Move `key` in the ordered list to the last
    fn update_key<Q: ?Sized>(list: &mut VecDeque<Key>, key: &Q)
    where
        Key: Borrow<Q>,
        Q: Ord,
    {
        if let Some(pos) = list.iter().position(|k| k.borrow() == key) {
            let _ = list.remove(pos).map(|it| list.push_back(it));
        }
    }

    /// If expiry timeout is set, removes expired items from the cache and returns them.
    fn remove_expired(&mut self) -> Vec<(Key, Value)> {
        let (map, list) = (&mut self.map, &mut self.list);

        if let Some(ttl) = self.time_to_live {
            let mut expired_values = Vec::new();
            for key in list.iter() {
                if map[key].1 + ttl >= Instant::now() {
                    break;
                }
                if let Some(entry) = map.remove(key) {
                    expired_values.push(entry.0);
                }
            }
            // remove keys as well
            return list
                .drain(..expired_values.len())
                .zip(expired_values)
                .collect();
        } else if map.is_empty() {
            list.clear();
        }

        Vec::new()
    }

    /// Removes least recently used items to make space for new ones.
    fn remove_lru(&mut self) {
        if self.map.len() >= self.capacity {
            for key in self.list.drain(..=self.map.len() - self.capacity) {
                assert!(self.map.remove(&key).is_some());
            }
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
    use super::*;
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

            let _ = lru_cache.insert(
                Temp {
                    id: generate_random_vec::<u8>(64),
                },
                i,
            );

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        sleep(101);
        let _ = lru_cache.insert(
            Temp {
                id: generate_random_vec::<u8>(64),
            },
            1,
        );

        assert_eq!(lru_cache.len(), 1);
    }

    mod notify_insert {
        use super::*;

        #[test]
        fn it_removes_expired_entries() {
            let ttl = Duration::from_millis(200);
            let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(ttl);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);
            sleep(250);

            let _ = lru_cache.notify_insert(3, 3);

            assert_eq!(lru_cache.map.len(), 1);
            assert_eq!(lru_cache.map[&3].0, 3);
        }

        #[test]
        fn it_returns_removed_expired_entries() {
            let ttl = Duration::from_millis(200);
            let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(ttl);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);
            sleep(250);

            let (_replaced, expired) = lru_cache.notify_insert(3, 3);

            assert_eq!(expired.len(), 2);
            assert_eq!(expired[0], (1, 1));
            assert_eq!(expired[1], (2, 2));
        }
    }

    mod iter {
        use super::*;

        #[test]
        fn it_returns_none_when_cache_is_empty() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(3);

            let next = lru_cache.iter().next();

            assert!(next.is_none());
        }

        #[test]
        fn it_updates_item_timestamps_of_traversed_items() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(3);
            let _ = lru_cache.insert(0, 0);
            sleep(1);
            let _ = lru_cache.insert(1, 1);
            sleep(1);
            let _ = lru_cache.insert(2, 2);
            sleep(1);

            let initial_instant0 = lru_cache.map[&0].1;
            let initial_instant2 = lru_cache.map[&2].1;
            sleep(1);

            // only the first two entries should have their timestamp updated (and position in list)
            let _ = lru_cache.iter().take(2).all(|_| true);

            assert_ne!(lru_cache.map[&2].1, initial_instant2);
            assert_eq!(lru_cache.map[&0].1, initial_instant0);
        }

        #[test]
        fn it_moves_traversed_items_to_the_top_of_the_cache() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(3);
            let _ = lru_cache.insert(0, 0);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);

            let _ = lru_cache.iter().take(2).all(|_| true);

            assert_eq!(*lru_cache.list.front().unwrap(), 0);
            assert_eq!(*lru_cache.list.back().unwrap(), 1);
        }

        #[test]
        fn it_yields_the_most_recent_items_first() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(4);
            let _ = lru_cache.insert(2, 2);
            let _ = lru_cache.insert(0, 0);
            let _ = lru_cache.insert(3, 3);
            let _ = lru_cache.insert(1, 1);

            let cached = lru_cache.iter().collect::<Vec<_>>();

            assert_eq!(cached, vec![(&1, &1), (&3, &3), (&0, &0), (&2, &2)]);
        }

        #[test]
        fn it_removes_expired_items() {
            let mut lru_cache =
                LruCache::<usize, usize>::with_expiry_duration(Duration::from_millis(3));
            let _ = lru_cache.insert(0, 0);
            sleep(1);
            let _ = lru_cache.insert(1, 1);
            sleep(4);
            let _ = lru_cache.insert(2, 2);

            let items: Vec<_> = lru_cache.iter().collect();

            assert_eq!(items.len(), 1);
            assert_eq!(items[0], (&2, &2));
        }
    }

    mod notify_iter {
        use super::*;

        #[test]
        fn it_returns_none_when_cache_is_empty() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(3);

            let next = lru_cache.notify_iter().next();

            assert!(next.is_none());
        }

        #[test]
        fn it_yields_the_most_recent_items_first() {
            let mut lru_cache = LruCache::<usize, usize>::with_capacity(4);
            let _ = lru_cache.insert(2, 2);
            let _ = lru_cache.insert(0, 0);
            let _ = lru_cache.insert(3, 3);
            let _ = lru_cache.insert(1, 1);

            let cached = lru_cache
                .notify_iter()
                .map(|entry| match entry {
                    TimedEntry::Valid(key, value) => (key, value),
                    _ => panic!("Unexpected expired entry"),
                })
                .collect::<Vec<_>>();

            assert_eq!(cached, vec![(&1, &1), (&3, &3), (&0, &0), (&2, &2)]);
        }

        #[test]
        fn it_produces_expired_and_valid_entries() {
            let mut lru_cache =
                LruCache::<usize, usize>::with_expiry_duration(Duration::from_millis(300));
            let _ = lru_cache.insert(0, 0);
            let _ = lru_cache.insert(1, 1);
            sleep(250);
            let _ = lru_cache.insert(2, 2);
            sleep(60);

            let expired: Vec<_> = lru_cache
                .notify_iter()
                .filter_map(|entry| match entry {
                    TimedEntry::Expired(key, value) => Some((key, value)),
                    _ => None,
                })
                .collect();
            let valid: Vec<_> = lru_cache
                .notify_iter()
                .filter_map(|entry| match entry {
                    TimedEntry::Valid(key, value) => Some((key, value)),
                    _ => None,
                })
                .collect();

            assert_eq!(valid.len(), 1);
            assert_eq!(valid[0], (&2, &2));
            assert_eq!(expired.len(), 2);
            assert_eq!(expired[0], (1, 1));
            assert_eq!(expired[1], (0, 0));
        }

        #[test]
        fn it_removes_expired_items() {
            let mut lru_cache =
                LruCache::<usize, usize>::with_expiry_duration(Duration::from_millis(300));
            let _ = lru_cache.insert(0, 0);
            let _ = lru_cache.insert(1, 1);
            sleep(250);
            let _ = lru_cache.insert(2, 2);
            sleep(60);

            let _items: Vec<_> = lru_cache.notify_iter().collect();

            assert!(lru_cache.get(&0).is_none());
            assert!(lru_cache.get(&1).is_none());
        }
    }

    #[test]
    fn peek_iter() {
        let time_to_live = Duration::from_millis(500);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        let _ = lru_cache.insert(0, 0);
        let _ = lru_cache.insert(2, 2);
        let _ = lru_cache.insert(3, 3);

        sleep(300);
        assert_eq!(
            vec![(&0, &0), (&2, &2), (&3, &3)],
            lru_cache.peek_iter().collect::<Vec<_>>()
        );
        assert_eq!(Some(&2), lru_cache.get(&2));
        let _ = lru_cache.insert(1, 1);
        let _ = lru_cache.insert(4, 4);

        sleep(300);
        assert_eq!(
            vec![(&1, &1), (&2, &2), (&4, &4)],
            lru_cache.peek_iter().collect::<Vec<_>>()
        );

        sleep(300);
        assert!(lru_cache.is_empty());
    }

    #[test]
    fn update_time_check() {
        let time_to_live = Duration::from_millis(500);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        assert_eq!(lru_cache.len(), 0);
        let _ = lru_cache.insert(0, 0);
        assert_eq!(lru_cache.len(), 1);

        sleep(300);
        assert_eq!(Some(&0), lru_cache.get(&0));
        sleep(300);
        assert_eq!(Some(&0), lru_cache.peek(&0));
        sleep(300);
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

    mod remove_expired {
        use super::*;

        #[test]
        fn it_removes_expired_entries_from_the_map() {
            let ttl = Duration::from_millis(200);
            let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(ttl);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);
            sleep(150);
            let _ = lru_cache.insert(3, 3);
            let _ = lru_cache.insert(4, 4);
            sleep(60);

            let _ = lru_cache.remove_expired();

            assert_eq!(lru_cache.map.len(), 2);
            assert_eq!(lru_cache.map[&3].0, 3);
            assert_eq!(lru_cache.map[&4].0, 4);
        }

        #[test]
        fn it_removes_expired_entries_from_the_list() {
            let ttl = Duration::from_millis(200);
            let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(ttl);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);
            sleep(150);
            let _ = lru_cache.insert(3, 3);
            let _ = lru_cache.insert(4, 4);
            sleep(60);

            let _ = lru_cache.remove_expired();

            assert_eq!(lru_cache.list.len(), 2);
            assert_eq!(lru_cache.list[0], 3);
            assert_eq!(lru_cache.list[1], 4);
        }

        #[test]
        fn it_returns_expired_entries() {
            let ttl = Duration::from_millis(200);
            let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(ttl);
            let _ = lru_cache.insert(1, 1);
            let _ = lru_cache.insert(2, 2);
            sleep(150);
            let _ = lru_cache.insert(3, 3);
            sleep(60);

            let expired = lru_cache.remove_expired();

            assert_eq!(expired.len(), 2);
            assert_eq!(expired[0], (1, 1));
            assert_eq!(expired[1], (2, 2));
        }
    }
}
