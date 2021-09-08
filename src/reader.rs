use std::{fs::read_to_string, path::Path};

use crate::errors::{LoxInterpreterError, Result};

pub fn read_file<P: AsRef<Path>>(path: P) -> Result<String> {
    read_to_string(path).map_err(LoxInterpreterError::IOError)
}
