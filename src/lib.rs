/*  Copyright 2015 MaidSafe.net limited

    This MaidSafe Software is licensed to you under (1) the MaidSafe.net Commercial License,
    version 1.0 or later, or (2) The General Public License (GPL), version 3, depending on which
    licence you accepted on initial access to the Software (the "Licences").

    By contributing code to the MaidSafe Software, or to this project generally, you agree to be
    bound by the terms of the MaidSafe Contributor Agreement, version 1.0, found in the root
    directory of this project at LICENSE, COPYING and CONTRIBUTOR respectively and also
    available at: http://www.maidsafe.net/licenses

    Unless required by applicable law or agreed to in writing, the MaidSafe Software distributed
    under the GPL Licence is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS
    OF ANY KIND, either express or implied.

    See the Licences for the specific language governing permissions and limitations relating to
    use of the MaidSafe Software.                                                                 */

#![crate_name = "lru_time_cache"]
#![crate_type = "lib"]
#![doc(html_logo_url = "http://maidsafe.net/img/Resources/branding/maidsafe_logo.fab2.png",
       html_favicon_url = "http://maidsafe.net/img/favicon.ico",
              html_root_url = "http://dirvine.github.io/dirvine/lru_time_cache/")]
#![feature(std_misc)]
#![feature(old_io)]

//! #lru cache limited via size or time  
//! 

extern crate chrono;

use std::usize;
use std::collections;

pub struct LruCache<K, V> where K: PartialOrd + Ord + Clone {
    map: collections::BTreeMap<K, (V, chrono::DateTime<chrono::Local>)>,
    list: collections::VecDeque<K>,
    capacity: usize,
    time_to_live: chrono::duration::Duration,
}

impl<K, V> LruCache<K, V> where K: PartialOrd + Ord + Clone {
    pub fn with_capacity(capacity: usize) -> LruCache<K, V> {
        LruCache {
            map: collections::BTreeMap::new(),
            list: collections::VecDeque::new(),
            capacity: capacity,
            time_to_live: chrono::duration::MAX,
        }
    }

    pub fn with_expiry_duration(time_to_live: chrono::duration::Duration) -> LruCache<K, V> {
        LruCache {
            map: collections::BTreeMap::new(),
            list: collections::VecDeque::new(),
            capacity: usize::MAX,
            time_to_live: time_to_live,
        }
    }

    pub fn with_expiry_duration_and_capacity(time_to_live: chrono::duration::Duration, capacity: usize) -> LruCache<K, V> {
        LruCache {
            map: collections::BTreeMap::new(),
            list: collections::VecDeque::new(),
            capacity: capacity,
            time_to_live: time_to_live,
        }
    }

    pub fn add(&mut self, key: K, value: V) {
        if !self.map.contains_key(&key) {
            while self.check_time_expired() || self.map.len() == self.capacity {
                self.remove_oldest_element();
            }

            self.list.push_back(key.clone());
            self.map.insert(key, (value, chrono::Local::now()));
        }
    }

    pub fn get(&mut self, key: K) -> Option<&V> {
       let get_result = self.map.get(&key);

       if get_result.is_some() {
           let pos_in_list = self.list.iter().enumerate().find(|a| !(*a.1 < key || *a.1 > key)).unwrap().0;
           self.list.remove(pos_in_list);
           self.list.push_back(key.clone());
           Some(&get_result.unwrap().0)
       } else {
           None
       }
    }

    pub fn check(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    fn remove_oldest_element(&mut self) {
        let key = self.list.pop_front().unwrap();
        self.map.remove(&key).unwrap();
    }

    fn check_time_expired(&self) -> bool {
        if self.time_to_live == chrono::duration::MAX || self.map.len() == 0 {
            false
        } else {
            self.map.get(self.list.front().unwrap()).unwrap().1 + self.time_to_live < chrono::Local::now()
        }
    }
}

#[cfg(test)]
mod test {
    extern crate chrono;

    use super::LruCache;
    use std::old_io;

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

        for i in (0..1000).rev() {
            assert!(lru_cache.check(&(1000 - 1)));
            assert!(lru_cache.get(1000 - 1).is_some());
            assert_eq!(*lru_cache.get(1000 - 1).unwrap(), 1000 - 1);
        }
    }

    #[test]
    fn time_only() {
        let time_to_live = chrono::duration::Duration::milliseconds(100);
        let mut lru_cache = LruCache::<usize, usize>::with_expiry_duration(time_to_live);

        for i in 0..10 {
            assert_eq!(lru_cache.len(), i);
            lru_cache.add(i, i);
            assert_eq!(lru_cache.len(), i + 1);
        }

        old_io::timer::sleep(chrono::duration::Duration::milliseconds(100));
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
        let time_to_live = chrono::duration::Duration::milliseconds(100);
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

        old_io::timer::sleep(chrono::duration::Duration::milliseconds(100));
        lru_cache.add(1, 1);

        assert_eq!(lru_cache.len(), 1);
    }
}
