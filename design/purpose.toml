# REQ-data
This defines the "artifact-data" module, a self contained programming API for
deserializing, processing and reserializing artifacts from either strings or a
set of paths to files.

> This is a work in progress

This document intends to give a highly detailed design of
the data module with the goals of:
- simplicity: the types should not overlap in purpose
- robustness: all methods should have well defined inputs and outputs. In
  addition, rigourous fuzz testing should be designed in from the very
  beginning.
- speed: many of the slowest operations should now be done concurrently,
  such as file system IO.
- memory usage: reference counts should be used to conserve memory+runtime
  where possible.
- self contained: this module should not depend on any other artifact modules

# SPC-data
The control flow and high level architecture for deserializing and processing
artifact data are as follows:

```dot
digraph G {
    node [shape=box];
    [[dot:SPC-data-src]];
    [[dot:SPC-data-auto_partof]];
    [[dot:SPC-data-join]];
    [[dot:SPC-data-completeness]];

    subgraph cluster_start {
        {start [label="paths to parse"; shape=oval ]}
     }
    subgraph cluster_src {
        label="parse src code links";
        start -> "SPC-DATA-SRC";
    }
    subgraph cluster_artifacts {
        label="parse artifacts";
        start -> "[[dot:.raw]]";
        "[[dot:.raw]]" -> "[[dot:.names]]";
        "[[dot:.names]]" -> "SPC-DATA-AUTO_PARTOF";
    }

    subgraph cluster_join {
        label="final steps"
        // join main and branch
        "SPC-DATA-SRC" -> "SPC-DATA-JOIN";
        "SPC-DATA-AUTO_PARTOF" -> "SPC-DATA-JOIN";
        "SPC-DATA-JOIN" -> "SPC-DATA-COMPLETENESS"
        "SPC-DATA-COMPLETENESS" -> "[[dot:.combine]]" -> done;
    }

    label="lints";
    // after main join
    "SPC-DATA-JOIN" -> [[dot:SPC-data-lint-subnames]];
    "[[dot:.names]]" -> [[dot:SPC-data-lint-text]];
    "SPC-DATA-JOIN" -> [[dot:SPC-data-lint-src]];
}
```

The following are major design choices:
- [[SPC-data-join]]: the general parallization architecture.
- [[SPC-data-cache]]: the "global" caching architecture.
- [[TST-data]]: the overall testing architecture

There are the following subparts, which are also linked in the graph above:
- [[SPC-data-src]]: "deserialize" the source code and extract the links to
  artifacts
- [[.raw]]: deserialize the artifact files into "raw" data.
- [[.names]]: deserialize the artifact names into objects.
- [[SPC-data-auto_partof]]: Determine the auto-partofs into a Map.
- [[SPC-data-completeness]]: Calculate implemented% and tested%.
- [[.combine]]: combine to create artifacts.

In addition:
- [[SPC-data-lint]]: specified lints
- [[SPC-data-ser]]: serialization specification

# SPC-data-auto_partof
TODO

# SPC-data-cache
The data cache has the following goals:
- save on memory
- reduce operating system calls
- minor savings on speed

The cached datatypes are:
- `Name`: `Arc<InternalName>`
- `PathRc`: `Arc<PathBuf>`

There are two primary caches:
- [[.name]]: the name cache is a `HashMap<String.to_ascii_uppercase, Name>`.
  - Names are looked up by simply capitalizing all their characters. 
  - If they do not exist then they are validated and inserted.
  - This saves on validating the names if they already exist
  - It also means that the first name that is insertd will be the "standard"
    raw-representation of that name. So if you define a name as `ART-name`
    but then use it in a `partof` as `ART-NAME`, `art fmt` will correct
    them all to `ART-name`.
  - The `CachedNames` object has the following methods:
    - `get(name: &str) -> Result<Name>`: return the cached Name object.
- [[.path]]: the path cache has two components:
    - `HashSet<PathRc>` reference counted set of all known paths.
    - `HashMap<PathBuf, PathRc>`: this contains a memoized reference of all
      input paths and their `canonicalize()` path (which requires an OS call).
    - The `CachedPaths` object has the following methods:
        - `get(path: &Path) -> Result<PathRc>`: return the canonicalized path in
          the fastest way possible.

# SPC-data-completeness
TODO

# SPC-data-join
TODO

# SPC-data-lint
TODO

# SPC-data-lint-src
TODO

# SPC-data-lint-subnames
TODO

# SPC-data-lint-text
TODO

# SPC-data-ser
TODO

# SPC-data-src
TODO

# TST-data
Testing the data deserialization and processing, as well as reserialization is a major
concern. The `data` API is used for:
- Loading artifacts at init time.
- Formatting artifacts and dumping them to files (toml, markdown, etc)
- Editing artifacts through the web-ui and revalidating them before dumping them.
- Exporting the artifact as JSON, both for the web-ui and for external tools.

The primary approaches to testing shall be:
- Sanity tests: every data type will have ultra simple human written
  "sanity" tests to verify that they work according to user input.
- [[TST-data-fuzz]]: scaleable fuzz testing design
- [[TST-data-interop]]: interop testing strategy.

# TST-data-fuzz
All data objects shall be designed from the beginning to be fuzz tested, so
that even complex "projects" can be built up with random collections of
artifacts in differing states.

Obviously this will also allow for fast fuzz testing of the smaller objects themselves.

The old API used `Type::fake()` in lots of places -- these are good flags for *some* of
the places that fuzz testing could have been used instead (but the examples are much
larger than just that).

The major workhorse here will be the [quickcheck][1] library. The following datatypes
will have `Abitrary` implemented for them and releated tests performed against them:
- `Name` (and by extension `Partof`)
- `InvalidName`
- `RawArtifact`
  - `Done`
  - `CodeRef`
  - `CodeLoc`
  - `Text`
- `RawCodeLoc`: simply a file with given code references inserted at random.
- `HashMap<Name, RawArtifact>`

From the implementations, we can randomize testing for the following:
- [[.load_name]]: use `Name` and `InvalidName` to great effect.
- [[.load_artifacts]]: simply convert randomly generated artifacts into files
- [[.load_src]]: load RawCodeLoc and have expected result
  

[1]: https://docs.rs/quickcheck/0.4.2/quickcheck/

# TST-data-interop
There shall be a "interop test harness" constructed for doing interop testing.
The basic design is:
- *Each test is a full project* (folder with a `.art` folder, source files and
  design documents).
- Each test contains assertions at `path/to/test/art-assertions.toml`
- The assertions file contains:
    - artifact_paths: a list of all paths that should have been loaded for
      artifacts.
    - source_paths: a list of all paths that should have been loaded for
      source code.
    - failing-lints: dict of failing lints by each lint's subgroup.
    - artifacts: list of "artifact objects", where each object can be
      represented as fully as is desired (every attribute of ArtifactData can
      be represented)
- The test harness then loads the project and assertions file and asserts all
  of the assertions.
