use core::cmp::PartialEq;
use core::fmt;

use crate::{Domain, Suffix, Type};

impl<'a> fmt::Display for Suffix<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl<'a> fmt::Display for Domain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.to_str())
    }
}

impl Default for Type {
    fn default() -> Self {
        Type::Icann
    }
}

impl<'a, 'b> PartialEq<&'a str> for Domain<'b> {
    fn eq(&self, other: &&'a str) -> bool {
        self.to_str() == *other
    }
}

impl<'a, 'b> PartialEq<&'a str> for Suffix<'b> {
    fn eq(&self, other: &&'a str) -> bool {
        self.to_str() == *other
    }
}
