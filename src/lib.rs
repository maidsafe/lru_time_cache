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
//! extern crate time;
//! use ::lru_time_cache::LruCache;
//!
//! # fn main() {
//! // Construct an `LruCache` of `<u8, String>`s, limited by key count
//! let max_count = 10;
//! let lru_cache = LruCache::<u8, String>::with_capacity(max_count);
//!
//! // Construct an `LruCache` of `<String, i64>`s, limited by expiry time
//! let time_to_live = ::time::Duration::milliseconds(100);
//! let lru_cache = LruCache::<String, i64>::with_expiry_duration(time_to_live);
//!
//! // Construct an `LruCache` of `<u64, Vec<u8>>`s, limited by key count and expiry time
//! let lru_cache = LruCache::<u64, Vec<u8>>::with_expiry_duration_and_capacity(time_to_live,
//!                                                                             max_count);
//! # }
//! ```

#![doc(html_logo_url =
           "https://raw.githubusercontent.com/maidsafe/QA/master/Images/maidsafe_logo.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
       html_root_url = "http://maidsafe.github.io/lru_time_cache")]

#![forbid(
    bad_style,              // Includes:
                            // - non_camel_case_types:   types, variants, traits and type parameters
                            //                           should have camel case names,
                            // - non_snake_case:         methods, functions, lifetime parameters and
                            //                           modules should have snake case names
                            // - non_upper_case_globals: static constants should have uppercase
                            //                           identifiers
    exceeding_bitshifts,    // shift exceeds the type's number of bits
    mutable_transmutes,     // mutating transmuted &mut T from &T may cause undefined behavior
    no_mangle_const_items,  // const items will not have their symbols exported
    unknown_crate_types,    // unknown crate type found in #[crate_type] directive
    warnings                // mass-change the level for lints which produce warnings
    )]

#![deny(
    deprecated,                    // detects use of #[deprecated] items
    drop_with_repr_extern,         // use of #[repr(C)] on a type that implements Drop
    improper_ctypes,               // proper use of libc types in foreign modules
    missing_docs,                  // detects missing documentation for public members
    non_shorthand_field_patterns,  // using `Struct { x: x }` instead of `Struct { x }`
    overflowing_literals,          // literal out of range for its type
    plugin_as_library,             // compiler plugin used as ordinary library in non-plugin crate
    private_no_mangle_fns,         // functions marked #[no_mangle] should be exported
    private_no_mangle_statics,     // statics marked #[no_mangle] should be exported
    raw_pointer_derive,            // uses of #[derive] with raw pointers are rarely correct
    stable_features,               // stable features found in #[feature] directive
    unconditional_recursion,       // functions that cannot return without calling themselves
    unknown_lints,                 // unrecognized lint attribute
    unsafe_code,                   // usage of `unsafe` code
    unused,                        // Includes:
                                   // - unused_imports:     imports that are never used
                                   // - unused_variables:   detect variables which are not used in
                                   //                       any way
                                   // - unused_assignments: detect assignments that will never be
                                   //                       read
                                   // - dead_code:          detect unused, unexported items
                                   // - unused_mut:         detect mut variables which don't need to
                                   //                       be mutable
                                   // - unreachable_code:   detects unreachable code paths
                                   // - unused_must_use:    unused result of a type flagged as
                                   //                       #[must_use]
                                   // - unused_unsafe:      unnecessary use of an `unsafe` block
                                   // - path_statements: path statements with no effect
    unused_allocation,             // detects unnecessary allocations that can be eliminated
    unused_attributes,             // detects attributes that were not used by the compiler
    unused_comparisons,            // comparisons made useless by limits of the types involved
    unused_features,               // unused or unknown features found in crate-level #[feature]
                                   // directives
    unused_parens,                 // `if`, `match`, `while` and `return` do not need parentheses
    while_true                     // suggest using `loop { }` instead of `while true { }`
    )]

#![warn(
    trivial_casts,            // detects trivial casts which could be removed
    trivial_numeric_casts,    // detects trivial casts of numeric types which could be removed
    unused_extern_crates,     // extern crates that are never used
    unused_import_braces,     // unnecessary braces around an imported item
    unused_qualifications,    // detects unnecessarily qualified names
    unused_results,           // unused result of an expression in a statement
    variant_size_differences  // detects enums with widely varying variant sizes
    )]

#![allow(
    box_pointers,                  // use of owned (Box type) heap memory
    fat_ptr_transmutes,            // detects transmutes of fat pointers
    missing_copy_implementations,  // detects potentially-forgotten implementations of `Copy`
    missing_debug_implementations  // detects missing implementations of fmt::Debug
    )]

#[cfg(test)]
extern crate rand;
extern crate time;

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

/// Implementation of [LRU cache](index.html#least-recently-used-lru-cache).
#[derive(Clone)]
pub struct LruCache<Key, Value> {
    map: ::std::collections::BTreeMap<Key, (Value, time::SteadyTime)>,
    list: ::std::collections::VecDeque<Key>,
    capacity: usize,
    time_to_live: time::Duration,
}

impl<Key, Value> LruCache<Key, Value> where Key: PartialOrd + Ord + Clone, Value: Clone {
    /// Constructor for capacity based `LruCache`.
    pub fn with_capacity(capacity: usize) -> LruCache<Key, Value> {
        LruCache {
            map: ::std::collections::BTreeMap::new(),
            list: ::std::collections::VecDeque::new(),
            capacity: capacity,
            time_to_live: time::Duration::max_value(),
        }
    }

    /// Constructor for time based `LruCache`.
    pub fn with_expiry_duration(time_to_live: time::Duration) -> LruCache<Key, Value> {
        LruCache {
            map: ::std::collections::BTreeMap::new(),
            list: ::std::collections::VecDeque::new(),
            capacity: ::std::usize::MAX,
            time_to_live: time_to_live,
        }
    }

    /// Constructor for dual-feature capacity and time based `LruCache`.
    pub fn with_expiry_duration_and_capacity(time_to_live: time::Duration,
                                             capacity: usize)
                                             -> LruCache<Key, Value> {
        LruCache {
            map: ::std::collections::BTreeMap::new(),
            list: ::std::collections::VecDeque::new(),
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

        self.map.insert(key, (value, time::SteadyTime::now())).map(|pair| pair.0)
    }

    /// Removes a key-value pair from the cache.
    pub fn remove(&mut self, key: &Key) -> Option<Value> {
        let result = self.map.remove(key);

        if result.is_some() {
            let position = self.list.iter().enumerate().find(|a| !(*a.1 < *key || *a.1 > *key))
                              .unwrap().0;
            let _ = self.list.remove(position);
            Some(result.unwrap().0)
        } else {
            None
        }
    }

    /// Retrieves a reference to the value stored under `key`, or `None` if the key doesn't exist.
    /// Also removes expired elements.
    pub fn get(&mut self, key: &Key) -> Option<&Value> {
        self.remove_expired();
        let list = &mut self.list;

        self.map.get(key).map(|result| {
            Self::update_key(list, key);
            &result.0
        })
    }

    /// Retrieves a mutable reference to the value stored under `key`, or `None` if the key doesn't
    /// exist.  Also removes expired elements.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Value> {
        self.remove_expired();
        let list = &mut self.list;

        self.map.get_mut(key).map(|result| {
            Self::update_key(list, key);
            &mut result.0
        })
    }

    /// Returns whether `key` exists in the cache or not.  Also removes expired elements.
    pub fn contains_key(&mut self, key: &Key) -> bool {
        self.remove_expired();
        self.map.contains_key(key)
    }

    /// Returns the size of the cache, i.e. the number of cached key-value pairs.  Also removes
    /// expired elements.
    pub fn len(&mut self) -> usize {
        self.remove_expired();
        self.map.len()
    }

    /// Returns a clone of all elements as an unordered vector of key-value tuples.  Also removes
    /// expired elements.
    // FIXME: We should really just implement the `iter` function for this Cache object, let the
    // user clone and collect the elements when needed.
    pub fn retrieve_all(&mut self) -> Vec<(Key, Value)> {
        self.remove_expired();
        let mut result = Vec::<(Key, Value)>::with_capacity(self.map.len());
        self.map.iter().all(|a| {
            result.push((a.0.clone(), a.1 .0.clone()));
            true
        });
        result
    }

    /// Returns a clone of all elements as a vector of key-value tuples ordered by most to least
    /// recently updated.  Also removes expired elements.
    pub fn retrieve_all_ordered(&mut self) -> Vec<(Key, Value)> {
        self.remove_expired();
        let mut result = Vec::<(Key, Value)>::with_capacity(self.list.len());
        for key in self.list.iter().rev() {
            match self.map.get(key) {
                Some(value) => result.push((key.clone(), value.0.clone())),
                None => (),
            }
        }
        result
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    pub fn entry(&mut self, key: Key) -> Entry<Key, Value> {
        // We need to do it the ugly way below due to this issue:
        // https://github.com/rust-lang/rfcs/issues/811
        //match self.get_mut(&key) {
        //    Some(value) => Entry::Occupied(OccupiedEntry{value: value}),
        //    None => Entry::Vacant(VacantEntry{key: key, cache: self}),
        //}
        if self.contains_key(&key) {
            Entry::Occupied(OccupiedEntry { value: self.get_mut(&key).unwrap() })
        } else {
            Entry::Vacant(VacantEntry { key: key, cache: self })
        }
    }

    fn remove_oldest_element(&mut self) {
        let _ = self.list.pop_front().map(|key| { assert!(self.map.remove(&key).is_some()) });
    }

    fn check_time_expired(&self) -> bool {
        if self.time_to_live == time::Duration::max_value() || self.map.len() == 0 {
            false
        } else {
            self.map.get(self.list.front().unwrap()).unwrap().1 + self.time_to_live <
            time::SteadyTime::now()
        }
    }

    fn update_key(list: &mut ::std::collections::VecDeque<Key>, key: &Key) {
        let position = list.iter().enumerate().find(|a| !(*a.1 < *key || *a.1 > *key)).unwrap().0;
        let _ = list.remove(position);
        list.push_back(key.clone());
    }

    fn remove_expired(&mut self) {
        while self.check_time_expired() {
            self.remove_oldest_element();
        }
    }
}

impl<'a, Key: PartialOrd + Ord + Clone, Value: Clone> VacantEntry<'a, Key, Value> {
    /// Inserts a value
    pub fn insert(self, value: Value) -> &'a mut Value {
        let _ = self.cache.insert(self.key.clone(), value);
        self.cache.get_mut(&self.key).unwrap()
    }
}

impl<'a, Value: Clone> OccupiedEntry<'a, Value> {
    /// Converts the entry into a mutable reference to its value.
    pub fn into_mut(self) -> &'a mut Value {
        self.value
    }
}

impl<'a, Key: PartialOrd + Ord + Clone, Value: Clone> Entry<'a, Key, Value> {
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
    fn generate_random_vec<T>(len: usize) -> Vec<T>
        where T: ::rand::Rand {
        let mut vec = Vec::<T>::with_capacity(len);
        for _ in 0..len {
            vec.push(::rand::random::<T>());
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
        let time_to_live = ::time::Duration::milliseconds(100);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        ::std::thread::sleep_ms(100);
        let _ = lru_cache.insert(11, 11);

        assert_eq!(lru_cache.len(), 1);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i + 1);
            let _ = lru_cache.insert(i, i);
            assert_eq!(lru_cache.len(), i + 2);
        }
    }

    #[test]
    fn time_only_check() {
        let time_to_live = ::time::Duration::milliseconds(50);
        let mut lru_cache = super::LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        assert_eq!(lru_cache.len(), 0);
        let _ = lru_cache.insert(0, 0);
        assert_eq!(lru_cache.len(), 1);

        ::std::thread::sleep_ms(100);

        assert!(!lru_cache.contains_key(&0));
        assert_eq!(lru_cache.len(), 0);
    }

    #[test]
    fn time_and_size() {
        let size = 10usize;
        let time_to_live = ::time::Duration::milliseconds(100);
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

        ::std::thread::sleep_ms(100);
        let _ = lru_cache.insert(1, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[test]
    fn time_size_struct_value() {
        let size = 100usize;
        let time_to_live = ::time::Duration::milliseconds(100);

        #[derive(PartialEq, PartialOrd, Ord, Clone, Eq)]
        struct Temp {
            id: Vec<u8>,
        }

        let mut lru_cache =
            super::LruCache::<Temp, usize>::with_expiry_duration_and_capacity(time_to_live, size);

        for i in 0..1000 {
            if i < size {
                assert_eq!(lru_cache.len(), i);
            }

            let _ = lru_cache.insert(Temp { id: generate_random_vec::<u8>(64), }, i);

            if i < size {
                assert_eq!(lru_cache.len(), i + 1);
            } else {
                assert_eq!(lru_cache.len(), size);
            }
        }

        ::std::thread::sleep_ms(100);
        let _ = lru_cache.insert(Temp { id: generate_random_vec::<u8>(64), }, 1);

        assert_eq!(lru_cache.len(), 1);
    }

    #[test]
    fn retrieve_all() {
        let size = 10usize;
        let mut lru_cache = super::LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            let _ = lru_cache.insert(i, i);
        }

        let all = lru_cache.retrieve_all();
        assert_eq!(all.len(), lru_cache.map.len());

        assert!(all.iter().all(|a|
            lru_cache.contains_key(&a.0) && *lru_cache.get(&a.0).unwrap() == a.1));
    }

    #[test]
    fn retrieve_all_ordered() {
        let size = 10usize;
        let mut lru_cache = super::LruCache::<usize, usize>::with_capacity(size);

        for i in 0..10 {
            let _ = lru_cache.insert(i, i);
        }

        let all = lru_cache.retrieve_all_ordered();
        assert_eq!(all.len(), lru_cache.map.len());

        for i in all.iter().rev() {
            lru_cache.remove_oldest_element();
            assert!(!lru_cache.contains_key(&i.0) && lru_cache.get(&i.0).is_none());
        }
    }
}
