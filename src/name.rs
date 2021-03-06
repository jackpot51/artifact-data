//! #SPC-data-name
//!
//! This is the name module, the module for representing artifact names
//! and their global cache.


use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::fmt;
use std::io;
use std::result;
use serde::{self, Serialize, Deserialize, Serializer, Deserializer};
use regex::Regex;

use dev_prelude::*;

// EXPORTED TYPES AND FUNCTIONS

#[derive(Debug, Fail)]
pub enum NameError {
    #[fail(display = "{}", msg)]
    InvalidName {
        msg: String,
    },
}

#[derive(Clone, PartialEq, Eq)]
/// The atomically reference counted name, the primary one used by
/// this module.
pub struct Name {
    internal_name: Arc<InternalName>,
}

/// type of an `Artifact`
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Type {
    REQ,
    SPC,
    TST,
}

/// Internal Name object, use Name instead.
#[derive(Debug, Clone)]
pub struct InternalName {
    /// The artifact type, determined from the name prefix
    pub ty: Type,
    /// Capitalized form
    pub key: Arc<String>,
    /// Raw "user" form
    pub raw: String,
}

// CONSTANTS

/// The location in the name where the type is split at
/// ```
/// REQ-foo
///    ^
/// ```
pub const TYPE_SPLIT_LOC: usize = 3;

macro_rules! NAME_VALID_CHARS {
    () => { "A-Z0-9_" };
}

/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &'static str = concat!(
    "(?:REQ|SPC|TST)-(?:[",
    NAME_VALID_CHARS!(),
    "]+-)*(?:[",
    NAME_VALID_CHARS!(),
    "]+)",
);

lazy_static!{
    /// Valid name regular expression
    static ref NAME_VALID_RE: Regex = Regex::new(
        &format!("(?i)^{}$", NAME_VALID_STR)).unwrap();
}


// NAME METHODS

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&self.raw)
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> result::Result<Name, D::Error>
        where D: Deserializer<'de>
    {
        // FIXME: can this be str::deserialize?
        let s = String::deserialize(deserializer)?;
        Name::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl fmt::Debug for Name {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.internal_name)
    }
}

// Name is basically a smart pointer for internal_name
impl Deref for Name {
    type Target = InternalName;

    fn deref(&self) -> &InternalName {
       self.internal_name.as_ref()
    }
}


impl FromStr for Name {
    type Err = Error;
    fn from_str(raw: &str) -> Result<Name> {
        let mut cache = ::cache::NAME_CACHE.lock().expect("name cache poisioned");
        cache.get(raw)
    }
}

// INTERNAL NAME METHODS

impl InternalName {
    /// Get the raw str representation
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the "key" representation of the name.
    ///
    /// i.e. `"TST-FOO-BAR"`
    pub fn key_str(&self) -> &str {
        &self.key
    }
}


impl Hash for InternalName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // name is a hash of its type and key
        self.ty.hash(state);
        self.key.hash(state);
    }
}

impl PartialEq for InternalName {
    fn eq(&self, other: &InternalName) -> bool {
        self.ty == other.ty && self.key == other.key
    }
}

impl Eq for InternalName {}

impl Ord for InternalName {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.ty, other.ty) {
            (Type::REQ, Type::REQ)
            | (Type::SPC, Type::SPC)
            | (Type::TST, Type::TST) => self.key.cmp(&other.key),

            (Type::REQ, _) => Ordering::Greater,
            (_, Type::REQ) => Ordering::Less,
            (Type::SPC, Type::TST) => Ordering::Greater,
            (Type::TST, Type::SPC) => Ordering::Less,
        }
    }
}

impl PartialOrd for InternalName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

// TYPE METHODS

impl Type {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Type::REQ => "REQ",
            Type::SPC => "SPC",
            Type::TST => "TST",
        }
    }
}

// NAME CACHE METHODS

impl ::cache::NameCache {
    /// Get the name from the cache, inserting it if it doesn't exist
    ///
    /// This is the only way that names are created.
    fn get(&mut self, raw: &str) -> Result<Name> {
        // FIXME: I would like to use Arc for raw+name, but
        // Borrow<str> is not implemented for Arc<String>
        if let Some(n) = self.names.get(raw) {
            return Ok(n.clone());
        }

        if !NAME_VALID_RE.is_match(&raw) {
            let msg = format!("Name is invalid: {}", raw);
            return Err(NameError::InvalidName { msg: msg }.into());
        }

        // Get the cached reference counted key
        // note: names can have different capitalization
        let key = {
            let k = Arc::new(raw.to_ascii_uppercase());
            self.keys.entry(k).key().clone()
        };

        let ty = match &key[0..TYPE_SPLIT_LOC] {
            "REQ" => Type::REQ,
            "SPC" => Type::SPC,
            "TST" => Type::TST,
            _ => unreachable!(),
        };

        let name = Name {
            internal_name: Arc::new(InternalName {
                ty: ty,
                key: key,
                raw: raw.into(),
            }),
        };
        self.names.insert(raw.into(), name.clone());
        Ok(name)
    }
}
