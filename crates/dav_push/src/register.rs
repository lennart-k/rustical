use crate::Trigger;
use rustical_xml::{XmlDeserialize, XmlRootTag, XmlSerialize};

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
#[xml(ns = "crate::namespace::NS_DAVPUSH")]
pub struct WebPushSubscription {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub push_resource: String,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    // DAVx5 4.4.9 does not seem to use it yet
    pub content_encoding: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub subscription_public_key: SubscriptionPublicKey,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub auth_secret: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct SubscriptionPublicKey {
    #[xml(ty = "attr", rename = b"type")]
    ty: String,
    #[xml(ty = "text")]
    key: String,
}

#[derive(XmlDeserialize, Clone, Debug, PartialEq)]
pub struct SubscriptionElement {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub web_push_subscription: WebPushSubscription,
}

#[derive(XmlDeserialize, XmlSerialize, Clone, Debug, PartialEq)]
pub struct TriggerElement(#[xml(ty = "untagged", flatten)] Vec<Trigger>);

#[derive(XmlDeserialize, XmlRootTag, Clone, Debug, PartialEq)]
#[xml(root = b"push-register")]
#[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
pub struct PushRegister {
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub subscription: SubscriptionElement,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub expires: Option<String>,
    #[xml(ns = "rustical_dav::namespace::NS_DAVPUSH")]
    pub trigger: Option<TriggerElement>,
}

#[cfg(test)]
mod tests {
    use crate::{ContentUpdate, PropertyUpdate};

    use super::*;
    use rustical_dav::header::Depth;
    use rustical_xml::XmlDocument;

    #[test]
    fn test_xml_push_register() {
        let push_register = PushRegister::parse_str(
            r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <push-register xmlns="https://bitfire.at/webdav-push" xmlns:D="DAV:">
                <subscription>
                    <web-push-subscription>
                        <push-resource>https://up.example.net/yohd4yai5Phiz1wi</push-resource>
                        <content-encoding>aes128gcm</content-encoding>
                        <subscription-public-key type="p256dh">BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4</subscription-public-key>
                        <auth-secret>BTBZMqHH6r4Tts7J_aSIgg</auth-secret>
                    </web-push-subscription>
                </subscription>
                <trigger>
                    <content-update>
                        <D:depth>infinity</D:depth>
                    </content-update>
                    <property-update>
                        <D:depth>0</D:depth>
                        <D:prop>
                            <D:displayname/>
                            <D:owner/>
                        </D:prop>
                    </property-update>
                </trigger>
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
                        push_resource: "https://up.example.net/yohd4yai5Phiz1wi".to_owned(),
                        content_encoding: Some("aes128gcm".to_owned()),
                        subscription_public_key: SubscriptionPublicKey { ty: "p256dh".to_owned(), key: "BCVxsr7N_eNgVRqvHtD0zTZsEc6-VV-JvLexhqUzORcxaOzi6-AYWXvTBHm4bjyPjs7Vd8pZGH6SRpkNtoIAiw4".to_owned() },
                        auth_secret: "BTBZMqHH6r4Tts7J_aSIgg".to_owned()
                    }
                },
                expires: Some("Wed, 20 Dec 2023 10:03:31 GMT".to_owned()),
                trigger: Some(TriggerElement(vec![
                    Trigger::ContentUpdate(ContentUpdate(Depth::Infinity)),
                    Trigger::PropertyUpdate(PropertyUpdate(Depth::Zero)),
                ]))
            }
        )
    }
}
