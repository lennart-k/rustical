use rustical_dav::extensions::CommonPropertiesProp;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressObjectPropName")]
pub enum AddressObjectProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Getetag(String),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    Getcontenttype(&'static str),

    // CalDAV (RFC 4791)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressData(String),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressObjectPropWrapperName", untagged)]
pub enum AddressObjectPropWrapper {
    AddressObject(AddressObjectProp),
    Common(CommonPropertiesProp),
}
