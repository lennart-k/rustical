use derive_more::derive::From;
use serde::ser::SerializeMap;
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, From)]
pub struct TagList(Vec<String>);

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
