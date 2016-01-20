# Lru Time Cache - Change Log

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
