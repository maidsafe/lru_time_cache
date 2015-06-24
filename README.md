# Lru Time Cache

[![](https://img.shields.io/badge/Project%20SAFE-Approved-green.svg)](http://maidsafe.net/applications) [![](https://img.shields.io/badge/License-GPL3-green.svg)](https://github.com/maidsafe/lru_time_cache/blob/master/COPYING)

**Primary Maintainer:** Chandra Prakash (prakash@maidsafe.net)

|Crate|Linux|Windows|OSX|Coverage|Issues|
|:------:|:-------:|:-------:|:-------:|:-------:|:-------:|
|[![](http://meritbadge.herokuapp.com/lru_time_cache)](https://crates.io/crates/lru_time_cache)|[![Build Status](https://travis-ci.org/maidsafe/lru_time_cache.svg?branch=master)](https://travis-ci.org/maidsafe/lru_time_cache)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=lru_time_cache_win64_status_badge)](http://ci.maidsafe.net:8080/job/lru_time_cache_win64_status_badge/)|[![Build Status](http://ci.maidsafe.net:8080/buildStatus/icon?job=lru_time_cache_osx_status_badge)](http://ci.maidsafe.net:8080/job/lru_time_cache_osx_status_badge/)|[![Coverage Status](https://coveralls.io/repos/maidsafe/lru_time_cache/badge.svg)](https://coveralls.io/r/maidsafe/lru_time_cache)|[![Stories in Ready](https://badge.waffle.io/maidsafe/lru_time_cache.png?label=ready&title=Ready)](https://waffle.io/maidsafe/lru_time_cache)|


| [API Documentation - Master branch](http://maidsafe.github.io/lru_time_cache/) | [SAFENetwork System Documentation](http://systemdocs.maidsafe.net/) | [MaidSafe website](http://www.maidsafe.net) | [Safe Community site](https://forum.safenetwork.io) |

#Overview
Provides a Last Recently Used [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be limited by size or time, reordered to most recently seen.

#Todo Items

## [0.2.1] More API changes
- [ ] Implement `iter` function
- [ ] Remove `retrieve_all` function (in favor of the above)
- [ ] Remove `add` function (deprecated in favor of the `insert` function from v0.1.6)
