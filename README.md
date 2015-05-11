# lru_time_cache 

|Crate|Travis|Appveyor|Coverage|
|:-------:|:-------:|:------:|:------:|
|[![](http://meritbadge.herokuapp.com/lru_time_cache)](https://crates.io/crates/lru_time_cache)|[![Build Status](https://travis-ci.org/maidsafe/lru_time_cache.svg?branch=master)](https://travis-ci.org/maidsafe/lru_time_cache)|[![Build status](https://ci.appveyor.com/api/projects/status/xj1muadwnd1ysmmc/branch/master?svg=true)](https://ci.appveyor.com/project/dirvine/lru-time-cache-o9t28/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/lru_time_cache/badge.svg)](https://coveralls.io/r/maidsafe/lru_time_cache)|


| [ API Documentation](http://maidsafe.github.io/lru_time_cache/) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Overview 

Provides a Last Recently Used [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be limited by size or time, reordered to most recently seen.

#Todo

- [x] Implement add_key_value
- [x] Test add_key_value (time and size based tests)
- [x] Implement check
- [x] Test check (time and size based tests)
- [x] Implement get(key)
- [x] Test get (time and size based tests)
- [x] API version 0.8.0
- [x] Implement delete_key  
- [x] Test delete_key (time and size based tests)
- [x] API version 0.1.0
