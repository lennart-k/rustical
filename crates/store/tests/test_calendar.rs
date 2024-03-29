use rstest::rstest;
use rstest_reuse::{self, apply, template};
use rustical_store::calendar::CalendarStore;
use rustical_store::sqlite_store::create_test_store;
use rustical_store::toml_store::TomlCalendarStore;

const TIMEZONE: &str = include_str!("examples/timezone.ics");
const EVENT: &str = include_str!("examples/event.ics");

#[template]
#[rstest]
#[case::toml(async {TomlCalendarStore::test()})]
#[case::sqlite(async {create_test_store().await.unwrap() })]
async fn cal_store<CS: CalendarStore>(
    #[future(awt)]
    #[case]
    mut store: CS,
) {
}

#[apply(cal_store)]
#[tokio::test]
async fn test_init<CS: CalendarStore>(store: CS) {
    store.get_events("asd").await.unwrap();
}

#[apply(cal_store)]
#[tokio::test]
async fn test_create_event<CS: CalendarStore>(store: CS) {
    let mut store = store;
    store
        .insert_calendar(
            "test".to_owned(),
            rustical_store::calendar::Calendar {
                id: "test".to_owned(),
                name: Some("Test Calendar".to_owned()),
                owner: "Test User".to_owned(),
                timezone: Some(TIMEZONE.to_owned()),
                ..Default::default() // timezone: TIMEZONE.to_owned(),
            },
        )
        .await
        .unwrap();

    store
        .upsert_event("test".to_owned(), "asd".to_owned(), EVENT.to_owned())
        .await
        .unwrap();

    let event = store.get_event("test", "asd").await.unwrap();
    assert_eq!(event.get_ics(), EVENT);
    assert_eq!(event.get_uid(), "asd");
}
