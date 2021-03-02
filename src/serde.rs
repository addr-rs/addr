use crate::{dns, domain};
use core::convert::TryFrom;
use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

impl Serialize for domain::Name<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for domain::Name<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = <&str>::deserialize(deserializer)?;
        Self::try_from(input).map_err(|_| {
            let invalid = Unexpected::Str(input);
            Error::invalid_value(invalid, &"a domain name")
        })
    }
}

impl Serialize for dns::Name<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for dns::Name<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let input = <&str>::deserialize(deserializer)?;
        Self::try_from(input).map_err(|_| {
            let invalid = Unexpected::Str(input);
            Error::invalid_value(invalid, &"a DNS name")
        })
    }
}
