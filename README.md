# Lru Time Cache

[![](https://img.shields.io/badge/Project%20SAFE-Approved-green.svg)](http://maidsafe.net/applications) [![](https://img.shields.io/badge/License-GPL3-green.svg)](https://github.com/maidsafe/lru_time_cache/blob/master/COPYING)

**Primary Maintainer:** Fraser Hutchison (fraser.hutchison@maidsafe.net)

|Crate|Linux/OS X|Windows|Coverage|Issues|
|:---:|:--------:|:-----:|:------:|:----:|
|[![](http://meritbadge.herokuapp.com/lru_time_cache)](https://crates.io/crates/lru_time_cache)|[![Build Status](https://travis-ci.org/maidsafe/lru_time_cache.svg?branch=master)](https://travis-ci.org/maidsafe/lru_time_cache)|[![Build status](https://ci.appveyor.com/api/projects/status/15km1vxtg83qgvb5/branch/master?svg=true)](https://ci.appveyor.com/project/MaidSafe-QA/lru-time-cache/branch/master)|[![Coverage Status](https://coveralls.io/repos/maidsafe/lru_time_cache/badge.svg)](https://coveralls.io/r/maidsafe/lru_time_cache)|[![Stories in Ready](https://badge.waffle.io/maidsafe/lru_time_cache.png?label=ready&title=Ready)](https://waffle.io/maidsafe/lru_time_cache)|


| [API Documentation - master branch](http://maidsafe.net/lru_time_cache/master) | [SAFE Network System Documentation](http://systemdocs.maidsafe.net) | [MaidSafe website](http://maidsafe.net) | [SAFE Network Forum](https://forum.safenetwork.io) |
|:------:|:-------:|:-------:|:-------:|

# Overview

Implementation of a Least Recently Used [caching algorithm](http://en.wikipedia.org/wiki/Cache_algorithms) in a container which may be limited by size or time, ordered by most recently seen.

# Todo Items

## [0.3.0] More API changes
- [ ] Implement `iter` function
- [ ] Remove `retrieve_all` function (in favor of the above)
