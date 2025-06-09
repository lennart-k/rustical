use rustical_dav::extensions::{CommonPropertiesProp, SyncTokenExtensionProp};
use rustical_dav_push::DavPushExtensionProp;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumVariants, PropName)]
#[xml(unit_variants_ident = "AddressbookPropName")]
pub enum AddressbookProp {
    // WebDAV (RFC 2518)
    #[xml(ns = "rustical_dav::namespace::NS_DAV")]
    Displayname(Option<String>),

    // CardDAV (RFC 6352)
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookDescription(Option<String>),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedAddressData(SupportedAddressData),
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", skip_deserializing)]
    SupportedReportSet(SupportedReportSet),
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

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct AddressDataType {
    #[xml(ty = "attr")]
    pub content_type: &'static str,
    #[xml(ty = "attr")]
    pub version: &'static str,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
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

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub enum ReportMethod {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    AddressbookMultiget,
    SyncCollection,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedReportWrapper {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV")]
    report: ReportMethod,
}

// RFC 3253 section-3.1.5
#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedReportSet {
    #[xml(ns = "rustical_dav::namespace::NS_CARDDAV", flatten)]
    supported_report: &'static [SupportedReportWrapper],
}

impl Default for SupportedReportSet {
    fn default() -> Self {
        Self {
            supported_report: &[
                SupportedReportWrapper {
                    report: ReportMethod::AddressbookMultiget,
                },
                SupportedReportWrapper {
                    report: ReportMethod::SyncCollection,
                },
            ],
        }
    }
}
