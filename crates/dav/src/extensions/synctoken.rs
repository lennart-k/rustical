use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, PropName, EnumVariants, Debug)]
#[xml(unit_variants_ident = "SyncTokenExtensionPropName")]
pub enum SyncTokenExtensionProp {
    // Collection Synchronization (RFC 6578)
    #[xml(ns = "crate::namespace::NS_DAV")]
    SyncToken(String),

    // CalendarServer
    #[xml(ns = "crate::namespace::NS_CALENDARSERVER")]
    Getctag(String),
}

pub trait SyncTokenExtension {
    fn get_synctoken(&self) -> String;

    fn get_prop(
        &self,
        prop: &SyncTokenExtensionPropName,
    ) -> Result<SyncTokenExtensionProp, crate::Error> {
        Ok(match &prop {
            SyncTokenExtensionPropName::SyncToken => {
                SyncTokenExtensionProp::SyncToken(self.get_synctoken())
            }
            SyncTokenExtensionPropName::Getctag => {
                SyncTokenExtensionProp::Getctag(self.get_synctoken())
            }
        })
    }

    fn set_prop(&self, _prop: SyncTokenExtensionProp) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }

    fn remove_prop(&self, _prop: &SyncTokenExtensionPropName) -> Result<(), crate::Error> {
        Err(crate::Error::PropReadOnly)
    }
}
