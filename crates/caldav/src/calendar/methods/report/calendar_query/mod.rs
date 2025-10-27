use crate::Error;
use rustical_ical::CalendarObject;
use rustical_store::CalendarStore;

mod elements;
pub(crate) use elements::*;
mod comp_filter;

pub async fn get_objects_calendar_query<C: CalendarStore>(
    cal_query: &CalendarQueryRequest,
    principal: &str,
    cal_id: &str,
    store: &C,
) -> Result<Vec<CalendarObject>, Error> {
    let mut objects = store
        .calendar_query(principal, cal_id, cal_query.into())
        .await?;
    if let Some(filter) = &cal_query.filter {
        objects.retain(|object| filter.matches(object));
    }
    Ok(objects)
}

#[cfg(test)]
mod tests {
    use rustical_dav::xml::PropElement;
    use rustical_xml::XmlDocument;

    use crate::{
        calendar::methods::report::{
            ReportRequest,
            calendar_query::{
                CalendarQueryRequest, CompFilterElement, FilterElement, ParamFilterElement,
                PropFilterElement, TextMatchElement,
            },
        },
        calendar_object::{CalendarObjectPropName, CalendarObjectPropWrapperName},
    };

    #[test]
    fn calendar_query_7_8_7() {
        const INPUT: &str = r#"
            <?xml version="1.0" encoding="utf-8" ?>
            <C:calendar-query xmlns:C="urn:ietf:params:xml:ns:caldav">
                <D:prop xmlns:D="DAV:">
                    <D:getetag/>
                    <C:calendar-data/>
                </D:prop>
                <C:filter>
                <C:comp-filter name="VCALENDAR">
                    <C:comp-filter name="VEVENT">
                        <C:prop-filter name="ATTENDEE">
                            <C:text-match collation="i;ascii-casemap">mailto:lisa@example.com</C:text-match>
                            <C:param-filter name="PARTSTAT">
                                <C:text-match collation="i;ascii-casemap">NEEDS-ACTION</C:text-match>
                            </C:param-filter>
                        </C:prop-filter>
                    </C:comp-filter>
                </C:comp-filter>
                </C:filter>
            </C:calendar-query>
        "#;

        let report = ReportRequest::parse_str(INPUT).unwrap();
        let calendar_query: CalendarQueryRequest =
            if let ReportRequest::CalendarQuery(query) = report {
                query
            } else {
                panic!()
            };
        assert_eq!(
            calendar_query,
            CalendarQueryRequest {
                prop: rustical_dav::xml::PropfindType::Prop(PropElement(
                    vec![
                        CalendarObjectPropWrapperName::CalendarObject(
                            CalendarObjectPropName::Getetag,
                        ),
                        CalendarObjectPropWrapperName::CalendarObject(
                            CalendarObjectPropName::CalendarData(Default::default())
                        ),
                    ],
                    vec![]
                )),
                filter: Some(FilterElement {
                    comp_filter: CompFilterElement {
                        is_not_defined: None,
                        time_range: None,
                        prop_filter: vec![],
                        comp_filter: vec![CompFilterElement {
                            prop_filter: vec![PropFilterElement {
                                name: "ATTENDEE".to_owned(),
                                text_match: Some(TextMatchElement {
                                    collation: "i;ascii-casemap".to_owned(),
                                    negate_condition: None
                                }),
                                is_not_defined: None,
                                param_filter: vec![ParamFilterElement {
                                    is_not_defined: None,
                                    name: "PARTSTAT".to_owned(),
                                    text_match: Some(TextMatchElement {
                                        collation: "i;ascii-casemap".to_owned(),
                                        negate_condition: None
                                    }),
                                }],
                                time_range: None
                            }],
                            comp_filter: vec![],
                            is_not_defined: None,
                            name: "VEVENT".to_owned(),
                            time_range: None
                        }],
                        name: "VCALENDAR".to_owned()
                    }
                }),
                timezone: None,
                timezone_id: None
            }
        )
    }
}
