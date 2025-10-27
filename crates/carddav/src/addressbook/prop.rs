use rustical_dav::{
    extensions::{CommonPropertiesProp, SyncTokenExtensionProp},
    xml::SupportedReportSet,
};
use rustical_dav_push::DavPushExtensionProp;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};
use strum_macros::VariantArray;

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressbookPropName")]
pub enum AddressbookProp {
    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedAddressData(SupportedAddressData),
    #[xml(ns = "rustical_dav::namespace::NS_DAV", skip_deserializing)]
    SupportedReportSet(SupportedReportSet<ReportMethod>),
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    MaxResourceSize(i64),
}

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressbookPropWrapperName", untagged)]
pub enum AddressbookPropWrapper {
    Addressbook(AddressbookProp),
    SyncToken(SyncTokenExtensionProp),
    DavPush(DavPushExtensionProp),
    Common(CommonPropertiesProp),
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct AddressDataType {
    #[xml(ty = "attr")]
    pub content_type: &'static str,
    #[xml(ty = "attr")]
    pub version: &'static str,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct SupportedAddressData {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    address_data_type: &'static [AddressDataType],
}

impl Default for SupportedAddressData {
    fn default() -> Self {
        Self {
            address_data_type: &[
                AddressDataType {
                    content_type: "text/vcard",
                    version: "3.0",
                },
                AddressDataType {
                    content_type: "text/vcard",
                    version: "4.0",
                },
            ],
        }
    }
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq, VariantArray)]
pub enum ReportMethod {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookMultiget,
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    SyncCollection,
}
