use quick_xml::name::Namespace;
use std::collections::HashMap;
use std::str::FromStr;

pub mod de;
mod error;
pub mod se;
mod unparsed;
mod value;

pub use de::XmlDeserialize;
pub use de::XmlDocument;
pub use error::XmlError;
pub use se::XmlSerialize;
pub use se::XmlSerializeRoot;
pub use unparsed::Unparsed;
pub use value::{ParseValueError, ValueDeserialize, ValueSerialize};
pub use xml_derive::EnumUnitVariants;
pub use xml_derive::EnumVariants;
pub use xml_derive::XmlRootTag;

pub trait XmlRootTag {
    fn root_tag() -> &'static [u8];
    fn root_ns() -> Option<Namespace<'static>>;
    fn root_ns_prefixes() -> HashMap<Namespace<'static>, &'static [u8]>;
}

#[derive(Debug)]
pub struct FromStrError;

pub trait EnumVariants {
    const TAGGED_VARIANTS: &'static [(Option<Namespace<'static>>, &'static str)];

    // Returns all valid xml names including untagged variants
    fn variant_names() -> Vec<(Option<Namespace<'static>>, &'static str)>;
}

pub trait EnumUnitVariants: Sized {
    type UnitVariants: Into<(Option<Namespace<'static>>, &'static str)> + From<Self> + FromStr;
}
