use crate::{ContentUpdate, PropertyUpdate, SupportedTriggers, Transports, Trigger};
use rustical_dav::header::Depth;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, PropName, EnumVariants)]
#[xml(unit_variants_ident = "DavPushExtensionPropName")]
pub enum DavPushExtensionProp {
    // WebDav Push
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Transports(Transports),
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    Topic(String),
    #[xml(skip_deserializing)]
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    SupportedTriggers(SupportedTriggers),
}

pub trait DavPushExtension {
    fn get_topic(&self) -> String;

    fn supported_triggers(&self) -> SupportedTriggers {
        SupportedTriggers(vec![
            Trigger::ContentUpdate(ContentUpdate(Depth::One)),
            Trigger::PropertyUpdate(PropertyUpdate(Depth::One)),
        ])
    }

    fn get_prop(
        &self,
        prop: &DavPushExtensionPropName,
    ) -> Result<DavPushExtensionProp, rustical_dav::Error> {
        Ok(match &prop {
            DavPushExtensionPropName::Transports => {
                DavPushExtensionProp::Transports(Transports::default())
            }
            DavPushExtensionPropName::Topic => DavPushExtensionProp::Topic(self.get_topic()),
            DavPushExtensionPropName::SupportedTriggers => {
                DavPushExtensionProp::SupportedTriggers(self.supported_triggers())
            }
        })
    }

    fn set_prop(&self, _prop: DavPushExtensionProp) -> Result<(), rustical_dav::Error> {
        Err(rustical_dav::Error::PropReadOnly)
    }

    fn remove_prop(&self, _prop: &DavPushExtensionPropName) -> Result<(), rustical_dav::Error> {
        Err(rustical_dav::Error::PropReadOnly)
    }
}
