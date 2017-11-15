pub use dev_prelude::*;
pub use quickcheck::{Arbitrary, Gen, Rng, Testable, empty_shrinker};
use name::{Name};

/// Given list of `(name, parent)`, assert `name.parent() == parent`
///
/// FIXME: make this fully generic
pub fn assert_method<F>(method: F, values: &[(&str, Option<&str>)])
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

