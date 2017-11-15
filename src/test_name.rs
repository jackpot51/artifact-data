use prelude::*;
use dev_prelude::*;
use test_prelude::*;
use name::*;


use serde_json;

impl Arbitrary for Name {
    fn arbitrary<G: Gen>(g: &mut G) -> Name {
        {
            // prevent memory from balooning with
            // unheld references
            let mut cache = NAME_CACHE.lock().unwrap();
            cache.clear();
        }

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
            {
                // prevent memory from balooning with
                // unheld references
                let mut cache = NAME_CACHE.lock().unwrap();
                cache.clear();
            }
            let seed = Name::from_str(&raw).expect(
                &format!("invalid name after shrink: {:?}", raw));
            self.seed = seed.clone();
            Some(seed)
        } else {
            None
        }
    }
}

/// Assert that the name is valid
fn assert_names_valid(raw: &[&str]) {
    let errors = raw
        .iter()
        .map(|r| (r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(_) => None,
            Err(e) => Some(raw),
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
            Ok(n) => Some(raw),
            Err(_) => None,
        }).collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were valid but shouldn't have been:\n{:#?}", errors);
    }
}

#[test]
fn names_sanity() {
    assert_names_valid(&[
        "REQ-a",
        "REQ-a-b",
        "REQ-foo",
        "SPC-foo",
        "TST-foo",
        "TST-Foo",
        "TST-FoO",
        "tst-FOO",
        "tst-foo",
        "TST-bPRJM07msqpQ",
        "TST-bPRJM07msqpQ-p",
        "TST-bPRJM07msqpQ-pRM",
        "TST-bPRJM07msqpQ-pRMBtV",
        "TST-bPRJM07msqpQ-pRMBtV-HJmJOpEgFTI2p8zdEMpluTbnkepzdELxf5CntsW",
    ]);

    assert_names_invalid(&[
        "RSK-foo",
        "RSK-foo",
        "REQ",
        "REQ-",
        "REQ-a-",
        "REQ-a-b-",
        "REQ",
        "SPC",
        "hello",
        "",
    ]);
}

#[test]
fn name_cache_sanity() {
    let expected1 = "REQ-foo";
    let expected2 = "REQ-FOO";
    let name1 = Name::from_str(expected1).unwrap();
    let name2 = Name::from_str(expected2).unwrap();

    assert_eq!(name1, name2);
    assert_eq!(expected1, name1.raw);
    assert_eq!(expected2, name2.raw);
}

#[test]
fn serde_sanity() {
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
    let result = names.iter().map(|n| n.raw.clone()).collect::<Vec<_>>();
    assert_eq!(expected, result.as_slice());
}

quickcheck! {
    fn roundtrip(name: Name) -> bool {
        let repr = name.key_str();
        let from_repr = Name::from_str(&repr).unwrap();

        from_repr == name && repr == from_repr.key_str()
    }
}
