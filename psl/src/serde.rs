use crate::{Domain, List, Psl, Suffix};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl<'a> Serialize for Domain<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'a> Deserialize<'a> for Domain<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let input = <&'a str as Deserialize<'a>>::deserialize(deserializer)?;
        match List.domain(input) {
            Some(domain) => Ok(domain),
            None => {
                let invalid = Unexpected::Str(input);
                Err(Error::invalid_value(invalid, &"a domain name"))
            }
        }
    }
}

impl<'a> Serialize for Suffix<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'a> Deserialize<'a> for Suffix<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        let input = <&'a str as Deserialize<'a>>::deserialize(deserializer)?;
        match List.suffix(input) {
            Some(suffix) => Ok(suffix),
            None => {
                let invalid = Unexpected::Str(input);
                Err(Error::invalid_value(invalid, &"a domain suffix"))
            }
        }
    }
}
