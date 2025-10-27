use rustical_xml::XmlSerialize;
use strum::VariantArray;

// RFC 3253 section-3.1.5
#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct SupportedReportSet<T: XmlSerialize + 'static> {
    #[xml(flatten)]
    #[xml(ns = "crate::namespace::NS_DAV")]
    supported_report: Vec<ReportWrapper<T>>,
}

impl<T: XmlSerialize + Clone + 'static> SupportedReportSet<T> {
    #[must_use] pub fn new(methods: Vec<T>) -> Self {
        Self {
            supported_report: methods
                .into_iter()
                .map(|method| ReportWrapper { report: method })
                .collect(),
        }
    }

    pub fn all() -> Self
    where
        T: VariantArray,
    {
        Self::new(T::VARIANTS.to_vec())
    }
}

#[derive(Debug, Clone, XmlSerialize, PartialEq, Eq)]
pub struct ReportWrapper<T: XmlSerialize> {
    #[xml(ns = "crate::namespace::NS_DAV")]
    report: T,
}
