use crate::{CalDavPrincipalUri, calendar::resource::CalendarResource};
use rustical_dav::resource::Resource;
use rustical_store::auth::Principal;
use rustical_xml::XmlSerializeRoot;
use serde_json::from_str;

#[tokio::test]
async fn test_propfind() {
    let requests: Vec<_> = include_str!("./test_files/propfind.requests")
        .trim()
        .split("\n\n")
        .collect();
    let principals: Vec<Principal> =
        from_str(include_str!("./test_files/propfind.principals.json")).unwrap();
    let resources: Vec<CalendarResource> =
        from_str(include_str!("./test_files/propfind.resources.json")).unwrap();

    for principal in principals {
        for (request, resource) in requests.iter().zip(&resources) {
            let propfind = CalendarResource::parse_propfind(request).unwrap();

            let response = resource
                .propfind(
                    &format!("/caldav/principal/{}/{}", principal.id, resource.cal.id),
                    &propfind.prop,
                    propfind.include.as_ref(),
                    &CalDavPrincipalUri("/caldav"),
                    &principal,
                )
                .unwrap();
            let output = response
                .serialize_to_string()
                .unwrap()
                .trim()
                .replace("\r\n", "\n");
            insta::assert_snapshot!(output);
        }
    }
}
