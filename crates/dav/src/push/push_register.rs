use rustical_xml::{XmlDeserialize, XmlRootTag};

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[xml(ns = "crate::namespace::NS_DAVPUSH")]
pub struct WebPushSubscription {
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    pub push_resource: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct SubscriptionElement {
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    pub web_push_subscription: WebPushSubscription,
}

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq)]
#[xml(root = b"push-register")]
#[xml(ns = "crate::namespace::NS_DAVPUSH")]
pub struct PushRegister {
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    pub subscription: SubscriptionElement,
    #[xml(ns = "crate::namespace::NS_DAVPUSH")]
    pub expires: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustical_xml::XmlDocument;

    #[test]
    fn test_xml_push_register() {
        let push_register = PushRegister::parse_str(
            r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <push-register xmlns="https://bitfire.at/webdav-push">
                <subscription>
                    <web-push-subscription>
                        <push-resource>https://up.example.net/yohd4yai5Phiz1wi</push-resource>
                    </web-push-subscription>
                </subscription>
                <expires>Wed, 20 Dec 2023 10:03:31 GMT</expires>
            </push-register>
    "#,
        )
        .unwrap();
        assert_eq!(
            push_register,
            PushRegister {
                subscription: SubscriptionElement {
                    web_push_subscription: WebPushSubscription {
                        push_resource: "https://up.example.net/yohd4yai5Phiz1wi".to_owned()
                    }
                },
                expires: Some("Wed, 20 Dec 2023 10:03:31 GMT".to_owned())
            }
        )
    }
}
