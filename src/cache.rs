//! #SPC-data-cache
//!
//! This defines the caches, currently there are two:
//! - NameCache: cache of name tokens.
//! - PathCache: cache of absolute paths. The logic lives here
use prelude::*;
use dev_prelude::*;
use name::{Type, Name};

lazy_static!{
    /// global cache of names
    pub(crate) static ref NAME_CACHE: Mutex<NameCache> = Mutex::new(NameCache {
        names: HashMap::new(),
        keys: HashMap::new(),
    });
}


/// Global cache of names. Note: the methods live in `name.rs`.
///
/// #SPC-data-cache.name
pub(crate) struct NameCache {
    pub(crate) names: HashMap<String, Name>,
    // Use HashMap just for the entry API
    // also... this is equivalent when optimized
    pub(crate) keys: HashMap<Arc<String>, ()>,
}


/// Clear the internal caches.
///
/// Mosty used for tests to prevent memory from balooning.
pub fn clear_cache() {
    let mut cache = NAME_CACHE.lock().unwrap();
    cache.keys.clear();
    cache.names.clear();
}


