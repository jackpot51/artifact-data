use prelude::*;
use dev_prelude::*;
use test_prelude::*;
use name::*;

impl Arbitrary for InternalName {
    fn arbitrary<G: Gen>(g: &mut G) -> InternalName {
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
        InternalName::from_str(&raw).expect(
            &format!("invalid arbitrary name: {}", raw)
        )
    }

    fn shrink(&self) -> Box<Iterator<Item=Self>> {
        NameShrinker::new(self.clone())
    }
}


///Iterator which returns successive attempts to shrink the Name's seed
struct NameShrinker {
    seed: InternalName,
}

impl NameShrinker {
    fn new(seed: InternalName) -> Box<Iterator<Item=InternalName>> {
        let mut shrinker = NameShrinker {
            seed: seed,
        };
        // Always drop the first value
        shrinker.next();
        Box::new(shrinker)
    }
}


impl Iterator for NameShrinker {
    type Item = InternalName;
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
            let seed = InternalName::from_str(&raw).expect(
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
        .map(|r| (r, InternalName::from_str(r)))
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
        .map(|r| (r, InternalName::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(n) => Some(raw),
            Err(_) => None,
        }).collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were valid but shouldn't have been:\n{:#?}", errors);
    }
}

#[test]
fn names_re_sanity() {
    let invalid = &[
        "REQ",
        "REQ-",
        "REQ-a-",
    ];

    for v in invalid {
        assert!(!NAME_VALID_RE.is_match(v));
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

quickcheck! {
    fn roundtrip(name: InternalName) -> bool {
        let repr = name.key_string();
        let from_repr = InternalName::from_str(&repr).unwrap();

        from_repr == name && repr == from_repr.key_string()
    }
}
