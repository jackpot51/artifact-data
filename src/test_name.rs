use prelude::*;
use dev_prelude::*;
use test_prelude::*;
use name::{self, Name, Type};

use serde_json;

// HELPERS and TRAITS

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
        .map(|r| (r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(_) => None,
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

/// Given list of `(name, parent)`, assert `name.parent() == parent`
fn assert_method<F>(method: F, values: &[(&str, Option<&str>)])
        where F: Fn(&Name) -> Option<Name>
{
    let errors = values
        .iter()
        .map(|&(name, expected)|
            // convert the strings to actual names
            ( Name::from_str(name).unwrap()
            , expected.map(|n| Name::from_str(n).unwrap())
            )
        )
        .filter_map(|(name, expected)| {
            let result = method(&name);
            if result == expected {
                None
            } else {
                Some(format!("input={:?} expect={:?} result={:?}", name, expected, result))
            }
        }).collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The method had unexpected results:\n{:#?}", errors);
    }
}

// SANITY TESTS

#[test]
fn names_sanity() {
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
fn sanity_name_cache() {
    let expected1 = "REQ-foo";
    let expected2 = "REQ-FOO";
    let name1 = Name::from_str(expected1).unwrap();
    let name2 = Name::from_str(expected2).unwrap();

    assert_eq!(name1, name2);
    assert_eq!(expected1, name1.raw);
    assert_eq!(expected2, name2.raw);
}

#[test]
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

#[test]
fn sanity_parent() {
    assert_method(Name::parent, &[
        // no parents
        ("REQ-foo", None),
        ("TST-a", None),
        ("TST-23kjskljef32", None),

        // has parents
        ("REQ-a-b", Some("REQ-a")),
        ("REQ-A-B", Some("REQ-A")),
        ("REQ-aasdf-bbSdf-DES", Some("REQ-aasdf-bbSdf")),
    ]);
}

#[test]
fn sanity_auto_partof() {
    assert_method(Name::auto_partof, &[
        ("REQ-foo", None),
        ("REQ-a-b", None),
        ("REQ-A-B", None),

        ("spc-aasdf-bbSdf-DES", Some("REQ-aasdf-bbSdf-DES")),
        ("TSt-a", Some("SPC-a")),
        ("TST-23kjskljef32", Some("SPC-23kjskljef32")),
    ]);
}

quickcheck! {
    fn fuzz_name_roundtrip(name: Name) -> bool {
        name::clear_cache();
        let repr = name.key_str();
        let from_repr = Name::from_str(&repr).unwrap();

        from_repr == name && repr == from_repr.key_str()
    }

    fn fuzz_name_parent(name: Name) -> bool {
        name::clear_cache();
        // basically do the same thing but a sligtly different way
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        if items.len() > 2 {
            items.pop();
            let expected_raw = items.join("-");
            let expected = Name::from_str(&expected_raw).unwrap();
            let result = name.parent().unwrap();
            expected_raw == result.raw && expected == result
        } else {
            name.parent().is_none()
        }
    }

    fn fuzz_name_auto_partof(name: Name) -> bool {
        name::clear_cache();
        let ty = match name.ty {
            Type::REQ => return name.auto_partof().is_none(),
            Type::SPC => "REQ",
            Type::TST => "SPC",
        };
        let mut items = name.raw.split('-').map(|s| s.to_string()).collect::<Vec<_>>();
        items[0] = ty.into();
        let expected_raw = items.join("-");
        let expected = Name::from_str(&expected_raw).unwrap();
        let result = name.auto_partof().unwrap();
        expected_raw == result.raw && expected == result
    }
}
