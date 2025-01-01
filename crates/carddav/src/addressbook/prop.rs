use rustical_xml::XmlSerialize;

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
