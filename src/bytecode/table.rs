use std::collections::HashMap;

use super::values::LoxBytecodeValue;

/// Naive implementation of the [hash chapter](https://craftinginterpreters.com/hash-tables.html).
///
/// All custom logic is replaced by native and (hopefully) performant enough `std::collections::HashMap`.
#[derive(Default)]
pub struct LoxBytecodeTable {
    entries: HashMap<String, LoxBytecodeValue>,
}

impl LoxBytecodeTable {
    /// Tries to find an entry by key.
    pub fn find(&self, key: &str) -> Option<&LoxBytecodeValue> {
        self.entries.get(key)
    }

    /// Writes a (key, value) entry into the table, returning true if
    /// the key was already set.
    pub fn set(&mut self, key: String, value: LoxBytecodeValue) -> bool {
        self.entries.insert(key, value).is_some()
    }

    /// Tries to remove an entry from the table by key, returning true if
    /// the key was indeed set.
    pub fn delete(&mut self, key: &str) -> bool {
        self.entries.remove(key).is_some()
    }

    /// Get the number of entries in the table.
    pub fn size(&self) -> usize {
        self.entries.len()
    }
}
