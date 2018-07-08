//! Mozilla's Public Suffix List compiled down to very fast native code

#![no_std]
#![cfg_attr(feature = "dynamic", crate_type = "dylib")]

extern crate psl;
#[macro_use]
extern crate psl_codegen;

/// Access to the compiled native list
#[derive(Psl, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct List;

impl List {
    /// Creates an instance of a new list
    pub fn new() -> List { List }
}
