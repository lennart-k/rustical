use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AddressDataType {
    #[serde(rename = "@content-type")]
    pub content_type: String,
    #[serde(rename = "@version")]
    pub version: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedAddressData {
    #[serde(rename = "CARD:address-data-type", alias = "address-data-type")]
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

#[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Resourcetype {
    #[serde(rename = "CARD:addressbook", alias = "addressbook")]
    addressbook: (),
    collection: (),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum ReportMethod {
    AddressbookMultiget,
    SyncCollection,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ReportWrapper {
    #[serde(rename = "$value")]
    report: ReportMethod,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
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
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportedReportSet {
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
