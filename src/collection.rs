use serde::{de::Error, Deserialize, Serialize};
use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Collection {
    Collection(String),
    All,
}

impl Display for Collection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "collection:{}",
            match &self {
                Collection::Collection(c) => c,
                Collection::All => "*",
            }
        )
    }
}

impl Serialize for Collection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Collection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let s = s
            .strip_prefix("collection:")
            .ok_or_else(|| D::Error::custom("expected collection:name"))?;

        Ok(match s {
            "*" => Collection::All,
            _ => Collection::Collection(s.to_owned()),
        })
    }
}
