use {Domain, Suffix, Psl};
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use serde::de::{Error, Unexpected};

/// Access to the compiled native list
#[derive(Psl, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct List;

impl List {
    /// Creates an instance of a new list
    pub fn new() -> List { List }
}

impl<'a> Serialize for Domain<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Domain<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let input = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
        match List.domain(input) {
            Some(domain) => { Ok(domain) }
            None => {
                let invalid = Unexpected::Str(input);
                Err(Error::invalid_value(invalid, &"a domain name"))
            }
        }
    }
}

impl<'a> Serialize for Suffix<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for Suffix<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where D: Deserializer<'de>
    {
        let input = <&'de str as Deserialize<'de>>::deserialize(deserializer)?;
        match List.suffix(input) {
            Some(suffix) => { Ok(suffix) }
            None => {
                let invalid = Unexpected::Str(input);
                Err(Error::invalid_value(invalid, &"a domain suffix"))
            }
        }
    }
}
