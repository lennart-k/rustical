use derive_more::derive::From;
use serde::ser::SerializeMap;

use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<String>);

struct TagListVisitor;

impl<'de> Visitor<'de> for TagListVisitor {
    type Value = TagList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("TagList")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut tags = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            tags.push(key);
        }
        Ok(TagList(tags))
    }
}

impl<'de> Deserialize<'de> for TagList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(TagListVisitor)
    }
}

impl Serialize for TagList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for name in &self.0 {
            map.serialize_entry(&name, &())?;
        }
        map.end()
    }
}

impl TagList {
    pub fn inner(&self) -> &Vec<String> {
        &self.0
    }
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }
}
