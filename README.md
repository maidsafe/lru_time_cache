# lru_time_cache 

 Travis build and test status (Linux / OS/X (soon)) |Appveyor build and test status (Windows) | Code Coverage
|:---------------------------------:|:----------------------------------------:|:--------------------------------:|
|[![Build Status](https://travis-ci.org/dirvine/lru_time_cache.svg?branch=master)](https://travis-ci.org/dirvine/lru_time_cache)|[![Build status](https://ci.appveyor.com/api/projects/status/jsuo65sa631h0kav?svg=true)](https://ci.appveyor.com/project/dirvine/lru_time_cache)|[![Coverage Status](https://coveralls.io/repos/dirvine/lru_time_cache/badge.svg)](https://coveralls.io/r/dirvine/lru_time_cache)|


| [Documentation](http://dirvine.github.io/lru_time_cache/) | [MaidSafe System Documention](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Overview 

Provides a Last Recently Used [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be limited by size or time. As any element is accessed at all, it is reordered to most recently seen.

#Todo

- [x] Implement add_key_value
- [x] Test add_key_value (time and size based tests)
- [x] Implement check
- [x] Test check (time and size based tests)
- [x] Implement get(key)
- [x] Test get (time and size based tests)
- [x] API version 0.8.0
- [ ] Implement delete_key  
- [ ] Test delete_key (time and size based tests)
- [ ] API version 0.1.0
