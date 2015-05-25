# Lru Time Cache

**Primary Maintainer:** Chandra Prakash (prakash@maidsafe.net)

|Crate|Travis|Windows|OSX|Coverage|
|:------:|:-------:|:-------:|:-------:|:-------:|
|[![](http://meritbadge.herokuapp.com/lru_time_cache)](https://crates.io/crates/lru_time_cache)|[![Build Status](https://travis-ci.org/maidsafe/lru_time_cache.svg?branch=master)](https://travis-ci.org/maidsafe/lru_time_cache)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=lru_time_cache_win64_status_badge)](http://ci.maidsafe.net:8080/job/lru_time_cache_win64_status_badge/)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=lru_time_cache_osx_status_badge)](http://ci.maidsafe.net:8080/job/lru_time_cache_osx_status_badge/)|[![Coverage Status](https://coveralls.io/repos/maidsafe/lru_time_cache/badge.svg)](https://coveralls.io/r/maidsafe/lru_time_cache)|


| [ API Documentation](http://maidsafe.github.io/lru_time_cache/) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net/) | [MaidSafe web site](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Overview
Provides a Last Recently Used [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be limited by size or time, reordered to most recently seen.

#Todo Items

## [0.1.7] More API changes
- [ ] Rename `check` to `contains_key`
- [ ] Implement `iter` function
- [ ] Remove `retrieve_all` function (in favor of the above)
- [ ] Remove `add` function (deprecated in favor of the `insert` function from v0.1.6)
