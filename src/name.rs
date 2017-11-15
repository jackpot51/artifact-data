//! #SPC-data-name
//!
//! This is the name module, the module for representing artifact names
//! and their global cache.


use prelude::*;
use dev_prelude::*;
use regex::Regex;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::convert::AsRef;
use std::ops::Deref;

// EXPORTED TYPES AND FUNCTIONS

/// Get the name from the cache, inserting it if it doesn't exist
pub fn cached_name(name: &str) -> Result<Name> {
    NAME_CACHE.get(name)
}

#[derive(Debug, Fail)]
enum NameError {
    #[fail(display = "Name is invalid: {}", raw)]
    InvalidName {
        raw: String,
    },

    #[fail(display = "Name must start with REQ, SPC or TST: {}", raw)]
    InvalidType {
        raw: String,
    },
}

/// Global cache of names
pub struct NameCache {
    names: Mutex<HashMap<String, Name>>,
}

// #[derive(Serialize, Deserialize)]
// #[serde(with = "serde_name")]
#[derive(Clone)]
/// The atomically reference counted name, the primary one used by
/// this module.
pub struct Name {
    internal_name: Arc<InternalName>,
}

impl Deref for Name {
    type Target = InternalName;

    fn deref(&self) -> &InternalName {
       self.internal_name.as_ref()
    }
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
    pub key: Vec<String>,
    /// Raw "user" form
    pub raw: String,
}

// CONSTANTS

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
    pub static ref NAME_VALID_RE: Regex = Regex::new(
        &format!("(?i)^{}$", NAME_VALID_STR)).unwrap();

    /// global cache of names
    pub static ref NAME_CACHE: NameCache = NameCache {
        names: Mutex::new(HashMap::new()),
    };
}


// NAME METHODS

impl InternalName {
    /// Get the raw str representation
    pub fn as_str(&self) -> &str {
        &self.raw
    }

    /// Get the "key" representation of the name.
    ///
    /// i.e. `"TST-FOO-BAR"`
    pub fn key_string(&self) -> String {
        let mut out = self.ty.as_str().to_string();
        for n in &self.key {
            write!(out, "-{}", n).unwrap();
        }
        out
    }
}

impl FromStr for InternalName {
    type Err = Error;
    fn from_str(s: &str) -> Result<InternalName> {
        if !NAME_VALID_RE.is_match(s) {
            Err(NameError::InvalidName { raw: s.into() }.into())
        } else {
            Ok(InternalName {
                ty: Type::from_str(&s[0..3])?,
                key: s[4..].split('-').map(|i| i.to_ascii_uppercase()).collect(),
                raw: s.into(),
            })
        }
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

impl FromStr for Type {
    type Err = Error;
    fn from_str(s: &str) -> Result<Type> {
        match s.to_ascii_uppercase().as_str() {
            "REQ" => Ok(Type::REQ),
            "SPC" => Ok(Type::SPC),
            "TST" => Ok(Type::TST),
            _ => Err(NameError::InvalidType { raw: s.into()}.into()),
        }
    }
}

// NAME CACHE METHODS

impl NameCache {
    /// Get the name from the cache, inserting it if it doesn't exist
    pub fn get(&self, raw: &str) -> Result<Name> {
        let mut names = self.names.lock().expect("NameCache poisoned");
        // I'm pretty sure this is actually faster than the entry API
        // We would have to call `raw.to_string()` to do `names.entry`,
        // but that isn't necessar.
        if let Some(n) = names.get(raw) {
            return Ok(n.clone());
        }
        let name = Name {
            internal_name: Arc::new(InternalName::from_str(raw)?),
        };
        names.insert(raw.to_string(), name.clone());
        Ok(name)
    }

    #[allow(dead_code)]
    /// Clear the cache, mostly used in testing but provided in
    /// case it is needed
    ///
    /// Since all the internal items are just reference counted
    /// InternalNames, they will still exist
    pub fn clear(&self) {
        let mut names = self.names.lock().expect("NameCache poisoned");
        names.clear();
    }
}

/// Methods for serializing/deserializing names
mod serde_name {
    use super::{Name, cached_name};
    use serde::{self, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(name: &Name, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(&name.raw)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Name, D::Error>
        where D: Deserializer<'de>
    {
        let s = String::deserialize(deserializer)?;
        cached_name(&s).map_err(serde::de::Error::custom)
    }
}
