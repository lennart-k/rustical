use crate::{ContentUpdate, PropertyUpdate, SupportedTriggers, Transports, Trigger};
use rustical_dav::header::Depth;
use rustical_xml::{EnumVariants, PropName, XmlDeserialize, XmlSerialize};

#[derive(XmlDeserialize, XmlSerialize, PartialEq, Eq, Clone, PropName, EnumVariants, Debug)]
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

    fn vapid_public_key(&self) -> Option<&str> {
        None
    }

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
            DavPushExtensionPropName::Transports => DavPushExtensionProp::Transports(
                Transports::new(self.vapid_public_key().map(ToOwned::to_owned)),
            ),
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

#[cfg(test)]
mod tests {
    use super::{DavPushExtension, DavPushExtensionProp, DavPushExtensionPropName};
    use crate::Transports;
    use rustical_xml::{XmlRootTag, XmlSerialize, XmlSerializeRoot};

    struct TestCollection {
        vapid_public_key: Option<&'static str>,
    }

    impl DavPushExtension for TestCollection {
        fn get_topic(&self) -> String {
            "test-topic".to_owned()
        }
        fn vapid_public_key(&self) -> Option<&str> {
            self.vapid_public_key
        }
    }

    #[derive(XmlSerialize, XmlRootTag)]
    #[xml(root = "document")]
    struct Document {
        transports: Transports,
    }

    fn advertised_transports(collection: &impl DavPushExtension) -> String {
        let DavPushExtensionProp::Transports(transports) = collection
            .get_prop(&DavPushExtensionPropName::Transports)
            .expect("transports prop")
        else {
            panic!("expected the transports prop");
        };
        Document { transports }.serialize_to_string().unwrap()
    }

    #[test]
    fn get_prop_advertises_the_instances_vapid_key() {
        let with_key = TestCollection {
            vapid_public_key: Some("BNcRexamplekey"),
        };
        let out = advertised_transports(&with_key);
        assert!(out.contains("vapid-public-key"), "{out}");
        assert!(out.contains("BNcRexamplekey"), "{out}");
    }

    #[test]
    fn get_prop_omits_vapid_key_when_the_instance_has_none() {
        let without_key = TestCollection {
            vapid_public_key: None,
        };
        let out = advertised_transports(&without_key);
        assert!(out.contains("web-push"), "{out}");
        assert!(!out.contains("vapid-public-key"), "{out}");
    }
}
