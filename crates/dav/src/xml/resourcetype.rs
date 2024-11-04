use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct Resourcetype(pub &'static [&'static str]);

struct ResourcetypeVisitor;

impl<'de> Visitor<'de> for ResourcetypeVisitor {
    type Value = Resourcetype;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("TagList")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        while map.next_key::<String>()?.is_some() {}
        Ok(Resourcetype(&[]))
    }
}

impl<'de> Deserialize<'de> for Resourcetype {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(ResourcetypeVisitor)
    }
}

impl Serialize for Resourcetype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for entry in self.0 {
            map.serialize_entry(entry, &())?;
        }
        map.end()
    }
}
