# Lru Time Cache - Change Log

## [0.10.0]

- Move iterators into a separate module
- Split different test cases
- Update `LruCache::peek_iter()` order - most recently used items will be
  produced first.
- Fix edge cases related to time atomicity
- Fix atomicity of insert on entry
- Make library work in Rust stable 1.41.
- Use `next` to get the first element in the cache

## [0.9.0]

- API to get expired or pushed out items from the LRU
- Update `LruCache::iter()` order - most recently used items will be produced
  first.
- Update `rand` dependency

## [0.8.1]
- Update to dual license (MIT/BSD)

## [0.8.0]
- Use rust 1.22.1 stable / 2017-12-02 nightly
- rustfmt 0.9.0 and clippy-0.0.175

## [0.7.0]
- Use rust 1.19 stable / 2017-07-20 nightly
- rustfmt 0.9.0 and clippy-0.0.144
- Replace -Zno-trans with cargo check
- Make appveyor script using fixed version of stable
- Use cargo_install from QA

## [0.6.0]
- Add support for using fake clock.
- CI, README, rustfmt and clippy cleanups.

## [0.5.0]
- Add `iter` and remove obsolete `retrieve_all` methods.

## [0.4.0]
- Add `clear`, `peek` and `peek_iter` methods.

## [0.3.1]
- Fix arithmetic operation overflows.

## [0.3.0]
- Remove dependency on the time crate.
- Use std::time::Duration in the API

## [0.2.7]
- Updated dependencies.

## [0.2.6]
- Allow non-Clone Value types.

## [0.2.5]
- Update time to live when accessing elements.

## [0.2.4]
- Update deprecated item, replaced by `std::thread::sleep`.

## [0.2.3]
- Remove wildcard dependencies.

## [0.2.2]
- Removes expired values before accessing elements. Removed deprecated check method.

## [0.2.1]
- Provides a getter to fetch all key value pairs in order.
- Removed `add` function (deprecated in favor of the `insert` function from v0.1.6)

## [0.1.7 - 0.2.0]
- [#21] (https://github.com/maidsafe/lru_time_cache/issues/21) Enforced lint checks
- Rename `check` to `contains_key`

## [0.1.6] API additions
- Implement the `entry` function
- Implement the `insert` function as a replacement for `add` (with same semantics as Rust's standard `Map::insert` functions)
- Implement the `get_mut`

## [0.0.0 - 0.1.5] First implementation
- Implement add_key_value
- Test add_key_value (time and size based tests)
- Implement check
- Test check (time and size based tests)
- Implement get(key)
- Test get (time and size based tests)
- API version 0.8.0
- Implement delete_key
- Test delete_key (time and size based tests)
- API version 0.1.0
