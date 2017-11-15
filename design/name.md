# SPC-data-name
partof: SPC-data-cache
###
The `Name` type shall be the exported "key" of artifacts.  Internally it is
reference counted by a global cache, externally it exposes itself with
the following methods:
- `from_str(s)`: create or automatically load the name. It will always
  exist in the cache after this operation.
- `as_str()`: get the string representation of the name.
- `key_str()`: get the name's "key" representation

Internally the name is an Atomically reference counted pointer, meaning
that cloning it is extremely cheap.
