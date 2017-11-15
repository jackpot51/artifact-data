# SPC-data-name
partof:
- SPC-data-cache
- REQ-data-type
###
The `Name` type shall be the exported "key" of artifacts.  Internally it is
reference counted by a global cache, externally it exposes itself with
the following methods:
- `Name.ty`: get the name's type ([[REQ-data-type]])
- `Name.from_str(s)`: create or automatically load the name. It will always
  exist in the cache after this operation.
- `Name.as_str()`: get the string representation of the name.
- `Name.key_str()`: get the name's "key" representation

Internally the name is an Atomically reference counted pointer, meaning
that cloning it is extremely cheap.


# TST-data-name
The `Name` type is fairly low level with no dependencies, so interop testing
is not necessary.

- [[.sanity_valid]]: assert that names are valid in the general use case as well
  as edge cases (one element, more than one element, etc)
- [[.sanity_invalid]]: assert that names are invalid for all edge cases
  (extra `--`, `REQ` by itself, `REQ-`, `REQ-a-`, etc).
- [[.sanity_serde]]: do basic check that serde works with names.
- [[.sanity_parent]]: basic checks that `Name::parent()` works as expected.
- [[.sanity_auto_partof]]: basic checks that `Name::auto_partof()` works as expected.
- [[.fuzz]]: fuzz definitions shall be applied to be used both here and
  externally.
- [[.fuzz_name_roundtrip]]: check that any two names are equal if their keys
  are equal.
- [[.fuzz_parent]]: use the fuzzed name to determine the parent in a different
  way than the code and validate that they are identical
- [[.fuzz_auto_partof]]: use the fuzzed name to determine the auto_partof
  in a different way than the code and validate that they are identical
