use crate::Transports;
use rustical_xml::{EnumUnitVariants, EnumVariants, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Clone, EnumUnitVariants, EnumVariants)]
#[xml(unit_variants_ident = "DavPushExtensionPropName")]
pub enum DavPushExtensionProp {
    // WebDav Push
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Transports(Transports),
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Topic(String),
}

pub trait DavPushExtension {
    fn get_topic(&self) -> String;

    fn get_prop(
        &self,
        prop: &DavPushExtensionPropName,
    ) -> Result<DavPushExtensionProp, rustical_dav::Error> {
        Ok(match &prop {
            DavPushExtensionPropName::Transports => {
                DavPushExtensionProp::Transports(Default::default())
            }
            DavPushExtensionPropName::Topic => DavPushExtensionProp::Topic(self.get_topic()),
        })
    }

    fn set_prop(&self, _prop: DavPushExtensionProp) -> Result<(), rustical_dav::Error> {
        Err(rustical_dav::Error::PropReadOnly)
    }

    fn remove_prop(&self, _prop: &DavPushExtensionPropName) -> Result<(), rustical_dav::Error> {
        Err(rustical_dav::Error::PropReadOnly)
    }
}
