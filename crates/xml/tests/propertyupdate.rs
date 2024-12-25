use rustical_xml::{Unparsed, XmlDeserialize, XmlDocument, XmlRootTag};

#[test]
fn test_propertyupdate() {
    #[derive(XmlDeserialize)]
    struct SetPropertyElement<T: XmlDeserialize> {
        prop: T,
    }

    #[derive(XmlDeserialize)]
    struct TagName {
        #[xml(ty = "tag_name")]
        name: String,
    }

    #[derive(XmlDeserialize)]
    struct PropertyElement {
        #[xml(ty = "untagged")]
        property: TagName,
    }

    #[derive(XmlDeserialize)]
    struct RemovePropertyElement {
        prop: PropertyElement,
    }

    #[derive(XmlDeserialize)]
    enum Operation<T: XmlDeserialize> {
        Set(SetPropertyElement<T>),
        Remove(RemovePropertyElement),
    }

    #[derive(XmlDeserialize, XmlRootTag)]
    #[xml(root = b"propertyupdate")]
    struct PropertyupdateElement<T: XmlDeserialize> {
        #[xml(ty = "untagged", flatten)]
        operations: Vec<Operation<T>>,
    }

    let doc = PropertyupdateElement::<Unparsed>::parse_str(
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
