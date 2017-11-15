//! #SPC-data-name
//!
//! This module defines all operations around creating the key (called Name)
//! used in Artifact.
use test_prelude::*;
use name::{self, Name, Type};

use serde_json;

// HELPERS and TRAITS

/// #TST-data-name.fuzz: enable fuzz testing of the `Name`
impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Name {
        let size = g.size() + 2;
        let max_sub = g.gen_range(1, size);
        let num_cells = g.gen_range(1, size);
        let ty = g.choose(&["REQ", "SPC", "TST", "req", "sPc", "TsT"]).unwrap();
        let mut cells = (0..num_cells).map(|_| {
            let size = g.gen_range(1, max_sub + 1);
            String::from_iter(g.gen_ascii_chars().take(size))
        }).collect::<Vec<String>>();
        cells.insert(0, ty.to_string());
        let raw = cells.join("-");
        Name::from_str(&raw).expect(
            &format!("invalid arbitrary name: {}", raw)
        )
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        NameShrinker::new(self.clone())
    }
}


///Iterator which returns successive attempts to shrink the Name's seed
struct NameShrinker {
    seed: Name,
}

impl NameShrinker {
    fn new(seed: Name) -> Box<Iterator<Item=Name>> {
        let mut shrinker = NameShrinker {
            seed: seed,
        };
        // Always drop the first value
        shrinker.next();
        Box::new(shrinker)
    }
}


impl Iterator for NameShrinker {
    type Item = Name;
    fn next(&mut self) -> Option<Self::Item> {
        if self.seed.key.len() > 1 {
            // just modify the seed to be one element smaller
            let raw: String = {
                let mut raw: Vec<_> = self.seed.raw
                    .split("-")
                    .collect();
                // remove last element
                raw.pop();
                raw.join("-")
            };
            let seed = Name::from_str(&raw).expect(
                &format!("invalid name after shrink: {:?}", raw));
            self.seed = seed.clone();
            Some(seed)
        } else {
            None
        }
    }
}

/// Return a vector of the `raw` names
pub fn names_raw(names: &[Name]) -> Vec<String> {
    names.iter().map(|n| n.raw.clone()).collect()
}

/// Assert that the name is valid
fn assert_names_valid(raw: &[&str]) {
    let errors = raw
        .iter()
        .map(|r| (*r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(name) => {
                if raw == name.raw {
                    None
                } else {
                    panic!("raw was different: {} => {}", raw, name.raw);
                }
            },
            Err(_) => Some(raw),
        }).collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were not valid:\n{:#?}", errors);
    }
}


/// Assert that the name is valid
fn assert_names_invalid(raw: &[&str]) {
    let errors = raw
        .iter()
        .map(|r| (r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(_) => Some(raw),
            Err(_) => None,
        }).collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were valid but shouldn't have been:\n{:#?}", errors);
    }
}

// SANITY TESTS

#[test]
/// #TST-data-name.sanity_valid
fn sanity_names_valid() {
    assert_names_valid(&[
        "REQ-a",
        "REQ-a-b",
        "REQ-foo",
        "REQ-foo_bar",
        "SPC-foo",
        "TST-foo",
        "TST-Foo",
        "TST-FoO",
        "tst-FOO",
        "tst-foo",
        "TST-bPRJM_07msqpQ",
        "TST-bPRJM07msqpQ-pRMBtV-HJmJOpEgFTI2p8zdEMpluTbnkepzdELxf5CntsW",
    ]);
}

#[test]
/// #TST-data-name.sanity_valid
fn sanity_names_invalid() {
    assert_names_invalid(&[
        "RSK-foo",
        "REQ",
        "REQ-",
        "REQ-a-",
        "REQ-a--",
        "REQ-a-b-",
        "REQ--a",
        "REQ-a--b",
        "REQ-a--b-",
        "REQ-a.b",
        "REQ-a_b.",
        "REQ",
        "SPC",
        "TST",
        "hello",
        "",
        "a",
    ]);
}

#[test]
/// #TST-data-name.sanity_serde
fn sanity_serde_name() {
    let json = r#"["REQ-foo","REQ-FOO","REQ-bar","SPC-foo-bar","tst-foo-BAR"]"#;
    let expected = &[
        "REQ-foo",
        "REQ-FOO",
        "REQ-bar",
        "SPC-foo-bar",
        "tst-foo-BAR",
    ];
    assert_eq!(json, serde_json::to_string(expected).unwrap());
    let names: Vec<Name> = serde_json::from_str(&json).unwrap();
    let result = names_raw(&names);
    assert_eq!(expected, result.as_slice());
}

quickcheck! {
    /// #TST-data-name.sanity_auto_partof
    fn fuzz_name_key(name: Name) -> bool {
        ::cache::clear_cache();
        let repr = name.key_str();
        let from_repr = Name::from_str(&repr).unwrap();
        from_repr == name && repr == from_repr.key_str()
    }
}
