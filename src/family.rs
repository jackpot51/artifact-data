use dev_prelude::*;
use name::{Type, Name, TYPE_SPLIT_LOC};

impl Name {
    /// #SPC-data-family.parent
    /// The parent of the name. This must exist if not None for all
    /// artifats.
    pub fn parent(&self) -> Option<Name> {
        let loc = self.raw.rfind('-').expect("name.parent:rfind");
        if loc == TYPE_SPLIT_LOC {
            None
        } else {
            Some(Name::from_str(&self.raw[0..loc]).expect("name.parent:from_str"))
        }
    }

    /// #SPC-data-family.auto_partof
    /// The artifact that this COULD be automatically linked to.
    ///
    /// - REQ is not autolinked to anything
    /// - SPC is autolinked to the REQ with the same name
    /// - TST is autolinked to the SPC with the same name
    pub fn auto_partof(&self) -> Option<Name> {
        let ty = match self.ty {
            Type::REQ => return None,
            Type::SPC => Type::REQ,
            Type::TST => Type::SPC,
        };
        let mut out = String::with_capacity(self.raw.len());
        out.push_str(ty.as_str());
        out.push_str(&self.raw[TYPE_SPLIT_LOC..self.raw.len()]);
        Some(Name::from_str(&out).unwrap())
    }
}

#[cfg(test)]
mod test {
    use test_prelude::*;
    use name::{Type, Name};

    #[test]
    /// #TST-data-name.sanity_parent
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
    /// #TST-data-name.sanity_auto_partof
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
        /// #TST-data-name.fuzz_parent
        fn fuzz_name_parent(name: Name) -> bool {
            ::cache::clear_cache();
            // Basically do the same thing as the code but in a slightly different way
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

        /// #TST-data-name.fuzz_auto_partof
        fn fuzz_name_auto_partof(name: Name) -> bool {
            ::cache::clear_cache();
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
}
