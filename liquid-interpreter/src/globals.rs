use std::fmt;

use error::{Error, Result};
use itertools;
use value::Object;
use value::PathRef;
use value::Value;

/// Immutable view into a template's global variables.
pub trait Globals: fmt::Debug {
    /// Check if global variable exists.
    fn contains_global(&self, name: &str) -> bool;

    /// Enumerate all globals
    fn globals(&self) -> Vec<&str>;

    /// Check if variable exists.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn contains_variable(&self, path: PathRef) -> bool;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn try_get_variable<'a>(&'a self, path: PathRef) -> Option<&'a Value>;

    /// Access a variable.
    ///
    /// Notes to implementers:
    /// - Don't forget to reverse-index on negative array indexes
    /// - Don't forget about arr.first, arr.last.
    fn get_variable<'a>(&'a self, path: PathRef) -> Result<&'a Value>;
}

impl Globals for Object {
    fn contains_global(&self, name: &str) -> bool {
        self.contains_key(name)
    }

    fn globals(&self) -> Vec<&str> {
        self.keys().map(|s| s.as_ref()).collect()
    }

    fn contains_variable(&self, path: PathRef) -> bool {
        get_variable_option(self, path).is_some()
    }

    fn try_get_variable<'a>(&'a self, path: PathRef) -> Option<&'a Value> {
        get_variable_option(self, path)
    }

    fn get_variable<'a>(&'a self, path: PathRef) -> Result<&'a Value> {
        if let Some(res) = self.try_get_variable(path) {
            return Ok(res);
        } else {
            for cur_idx in 1..path.len() {
                let subpath_end = path.len() - cur_idx;
                let subpath = &path[0..subpath_end];
                if let Some(parent) = self.try_get_variable(subpath) {
                    let subpath = itertools::join(subpath.iter(), ".");
                    let requested = &path[subpath_end];
                    let available = itertools::join(parent.keys(), ", ");
                    return Err(Error::with_msg("Unknown index")
                        .context("variable", format!("{}", subpath))
                        .context("requested index", format!("{}", requested))
                        .context("available indexes", format!("{}", available)));
                }
            }

            let requested = path.get(0).expect("`Path` guarantees at least one element").to_str().into_owned();
            let available = itertools::join(self.keys(), ", ");
            return Err(Error::with_msg("Unknown variable")
                .context("requested variable", requested)
                .context("available variables", available));
        }
    }
}

fn get_variable_option<'o>(obj: &'o Object, path: PathRef) -> Option<&'o Value> {
    let mut indexes = path.iter();
    let key = indexes.next()?;
    let key = key.to_str();
    let value = obj.get(key.as_ref())?;

    indexes.fold(Some(value), |value, index| {
        let value = value?;
        value.get(index)
    })
}
