use serde::{
    de::{VariantAccess, Visitor},
    Deserialize,
};

#[derive(Debug, Clone, PartialEq)]
pub struct TagName(pub String);

impl From<TagName> for String {
    fn from(value: TagName) -> Self {
        value.0
    }
}

impl From<String> for TagName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct __Visitor;

        impl<'de> Visitor<'de> for __Visitor {
            type Value = TagName;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("tagname")
            }
            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::EnumAccess<'de>,
            {
                let (name, variant): (String, _) = data.variant()?;
                VariantAccess::unit_variant(variant)?;
                Ok(TagName(name))
            }
        }
        deserializer.deserialize_enum("doesn't matter", &[], __Visitor)
    }
}
