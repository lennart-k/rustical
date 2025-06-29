use rustical_xml::{Unparsed, XmlDeserialize, XmlDocument, XmlRootTag};

#[test]
fn test_propertyupdate() {
    #[derive(XmlDeserialize)]
    struct SetPropertyElement<T: XmlDeserialize> {
        #[allow(dead_code)]
        prop: T,
    }

    #[derive(XmlDeserialize)]
    struct TagName {
        #[xml(ty = "tag_name")]
        #[allow(dead_code)]
        name: String,
    }

    #[derive(XmlDeserialize)]
    struct PropertyElement {
        #[xml(ty = "untagged")]
        #[allow(dead_code)]
        property: TagName,
    }

    #[derive(XmlDeserialize)]
    struct RemovePropertyElement {
        #[allow(dead_code)]
        prop: PropertyElement,
    }

    #[derive(XmlDeserialize)]
    enum Operation<T: XmlDeserialize> {
        Set(SetPropertyElement<T>),
        #[allow(dead_code)]
        Remove(RemovePropertyElement),
    }

    #[derive(XmlDeserialize, XmlRootTag)]
    #[xml(root = b"propertyupdate")]
    struct PropertyupdateElement<T: XmlDeserialize> {
        #[xml(ty = "untagged", flatten)]
        #[allow(dead_code)]
        operations: Vec<Operation<T>>,
    }

    PropertyupdateElement::<Unparsed>::parse_str(
        r#"
         <propertyupdate>
            <set>
                <prop>
                    <displayname>Hello</displayname>
                </prop>
            </set>
            <remove>
                <prop>
                    <displayname />
                </prop>
            </remove>
         </propertyupdate>
     "#,
    )
    .unwrap();
}
