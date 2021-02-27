use crate::{domain, suffix, Domain, Suffix};
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for Domain<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for Domain<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = Deserialize::deserialize(deserializer)?;
        match domain(input) {
            Some(domain) if domain.as_bytes() == input => Ok(domain),
            _ => {
                let invalid = Unexpected::Bytes(input);
                Err(Error::invalid_value(invalid, &"a domain name"))
            }
        }
    }
}

impl Serialize for Suffix<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.as_bytes())
    }
}

impl<'de> Deserialize<'de> for Suffix<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = Deserialize::deserialize(deserializer)?;
        match suffix(input) {
            Some(suffix) if suffix.as_bytes() == input => Ok(suffix),
            _ => {
                let invalid = Unexpected::Bytes(input);
                Err(Error::invalid_value(invalid, &"a domain suffix"))
            }
        }
    }
}
