use serde::ser::SerializeMap;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Resourcetype(pub &'static [&'static str]);

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
