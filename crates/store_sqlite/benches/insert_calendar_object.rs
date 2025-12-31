use criterion::{Criterion, criterion_group, criterion_main};
use rustical_ical::{CalendarObject, CalendarObjectType};
use rustical_store::{Calendar, CalendarMetadata, CalendarStore};
use rustical_store_sqlite::tests::test_store_context;

fn benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let cal_store = runtime.block_on(async {
        let cal_store = test_store_context().await.cal_store;

        cal_store
            .insert_calendar(Calendar {
                meta: CalendarMetadata {
                    displayname: Some("Yeet".to_owned()),
                    order: 0,
                    description: None,
                    color: None,
                },
                principal: "user".to_owned(),
                id: "okwow".to_owned(),
                timezone_id: None,
                deleted_at: None,
                synctoken: 0,
                subscription_url: None,
                push_topic: "asd".to_owned(),
                components: vec![
                    CalendarObjectType::Event,
                    CalendarObjectType::Todo,
                    CalendarObjectType::Journal,
                ],
            })
            .await
            .unwrap();
        cal_store
    });

    let object = CalendarObject::from_ics(include_str!("ical_event.ics").to_owned(), None).unwrap();

    let batch_size = 1000;
    let objects: Vec<_> = std::iter::repeat_n(object.clone(), batch_size).collect();

    c.bench_function("put_batch", |b| {
        b.to_async(&runtime).iter(async || {
            // yeet
            cal_store
                .put_objects("user".to_owned(), "okwow".to_owned(), objects.clone(), true)
                .await
                .unwrap();
        });
    });

    c.bench_function("put_single", |b| {
        b.to_async(&runtime).iter(async || {
            // yeet
            for _ in 0..1000 {
                cal_store
                    .put_object("user".to_owned(), "okwow".to_owned(), object.clone(), true)
                    .await
                    .unwrap();
            }
        });
    });

    runtime
        .block_on(cal_store.delete_calendar("user", "okwow", false))
        .unwrap();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
