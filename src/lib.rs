#![allow(unused_imports)]

extern crate failure;
extern crate prelude;
extern crate regex;
extern crate serde;

#[macro_use] extern crate failure_derive;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate matches;
#[macro_use] extern crate lazy_static;


#[cfg(test)]
#[macro_use] extern crate quickcheck;

mod name;
mod dev_prelude;

// TEST

#[cfg(test)] mod test_prelude;
#[cfg(test)] mod test_name;
