#![no_std]
#![cfg_attr(feature = "dynamic", crate_type = "dylib")]

extern crate psl;
#[macro_use]
extern crate psl_codegen;

#[derive(Psl, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct List;

impl List {
    pub fn new() -> List { List }
}
