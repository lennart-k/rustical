use rstest::rstest;
use rstest_reuse::{self, apply, template};
use rustical_store::{CalendarObject, CalendarStore};
use rustical_store_sqlite::create_test_store;

const TIMEZONE: &str = include_str!("examples/timezone.ics");
const EVENT: &str = include_str!("examples/event.ics");

#[template]
#[rstest]
#[case::sqlite(async {create_test_store().await.unwrap() })]
async fn cal_store<CS: CalendarStore>(
    #[future(awt)]
    #[case]
    mut store: CS,
) {
}

#[apply(cal_store)]
#[tokio::test]
async fn test_init<CS: CalendarStore>(_store: CS) {
    // store.get_events("asd").await.unwrap();
}

#[apply(cal_store)]
#[tokio::test]
async fn test_create_event<CS: CalendarStore>(store: CS) {
    store
        .insert_calendar(rustical_store::Calendar {
            id: "test".to_owned(),
            displayname: Some("Test Calendar".to_owned()),
            principal: "testuser".to_owned(),
            timezone: Some(TIMEZONE.to_owned()),
            ..Default::default() // timezone: TIMEZONE.to_owned(),
        })
        .await
        .unwrap();

    let object = CalendarObject::from_ics("asd".to_owned(), EVENT.to_owned()).unwrap();
    store
        .put_object("testuser".to_owned(), "test".to_owned(), object, true)
        .await
        .unwrap();

    let event = store.get_object("testuser", "test", "asd").await.unwrap();
    assert_eq!(event.get_ics(), EVENT);
    assert_eq!(event.get_id(), "asd");
}
