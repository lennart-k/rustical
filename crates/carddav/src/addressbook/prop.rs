use rustical_xml::XmlSerialize;

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct AddressDataType {
    #[xml(ty = "attr")]
    pub content_type: String,
    #[xml(ty = "attr")]
    pub version: String,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedAddressData {
    // #[serde(rename = "CARD:address-data-type", alias = "address-data-type")]
    #[xml(flatten)]
    address_data_type: Vec<AddressDataType>,
}

impl Default for SupportedAddressData {
    fn default() -> Self {
        Self {
            address_data_type: vec![
                AddressDataType {
                    content_type: "text/vcard".to_owned(),
                    version: "3.0".to_owned(),
                },
                AddressDataType {
                    content_type: "text/vcard".to_owned(),
                    version: "4.0".to_owned(),
                },
            ],
        }
    }
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub enum ReportMethod {
    AddressbookMultiget,
    SyncCollection,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct ReportWrapper {
    #[xml(ty = "untagged")]
    report: ReportMethod,
}

#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedReportWrapper {
    report: ReportWrapper,
}

impl From<ReportMethod> for SupportedReportWrapper {
    fn from(value: ReportMethod) -> Self {
        Self {
            report: ReportWrapper { report: value },
        }
    }
}

// RFC 3253 section-3.1.5
#[derive(Debug, Clone, XmlSerialize, PartialEq)]
pub struct SupportedReportSet {
    #[xml(flatten)]
    supported_report: Vec<SupportedReportWrapper>,
}

impl Default for SupportedReportSet {
    fn default() -> Self {
        Self {
            supported_report: vec![
                ReportMethod::AddressbookMultiget.into(),
                ReportMethod::SyncCollection.into(),
            ],
        }
    }
}
