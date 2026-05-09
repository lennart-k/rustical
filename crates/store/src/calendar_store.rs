use crate::{CalendarView, CollectionMetadata, FullCalendarView, calendar::BaseCal, error::Error};
use async_trait::async_trait;
use chrono::NaiveDate;
use rustical_ical::CalendarObject;

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct CalendarQuery {
    pub time_start: Option<NaiveDate>,
    pub time_end: Option<NaiveDate>,
}

#[async_trait]
pub trait CalendarStore: Send + Sync + 'static {
    /// Return a principal's calendar view
    async fn get_calendar(
        &self,
        principal: &str,
        name: &str,
        show_deleted: bool,
    ) -> Result<FullCalendarView, Error>;

    /// List a principal's calendar views
    async fn get_calendars(&self, principal: &str) -> Result<Vec<FullCalendarView>, Error>;

    /// List a principal's deleted/rejected calendar views
    async fn get_deleted_calendars(&self, principal: &str) -> Result<Vec<FullCalendarView>, Error>;

    /// Updates calendar metadata
    /// Accepts the base calendar either by value or by reference
    /// Global properties (e.g. timezone, components) are only updated if the principal has
    /// appropriate access
    async fn update_calendar<C: BaseCal>(
        &self,
        principal: &str,
        name: &str,
        calendar: CalendarView<C>,
    ) -> Result<(), Error>;

    /// Inserts a calendar view and/or calendar.
    /// - If base calendar passed by reference:
    ///   - Create view on existing calendar
    /// - If base calendar passed by value:
    ///   - If owner and principal match: Create new calendar
    ///   - If owner and principal don't match: Reject
    async fn insert_calendar<C: BaseCal>(&self, calendar: CalendarView<C>) -> Result<(), Error>;

    /// Deletes a calendar view.
    /// - If use_trashbin=true:
    ///   - Delete only the view. If it's the owner's view, sharees will still see the calendar
    /// - If use_trashbin=false.
    ///   - If principal is owner: Delete the calendar and therefore all views
    ///   - If principal is not owner: Delete calendar view permanently. Sharee would need to
    ///     re-request access
    async fn delete_calendar(
        &self,
        principal: &str,
        name: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;

    /// Restore a calendar view
    async fn restore_calendar(&self, principal: &str, name: &str) -> Result<(), Error>;

    /// Import a calendar
    async fn import_calendar(
        &self,
        calendar: FullCalendarView,
        objects: Vec<CalendarObject>,
        merge_existing: bool,
    ) -> Result<(), Error>;

    // The methods below all act on the base calendar.
    // Whether they should reference the calendar view or th underlying calendar remains an open
    // question.

    async fn sync_changes(
        &self,
        principal: &str,
        cal_id: &str,
        synctoken: i64,
    ) -> Result<(Vec<(String, CalendarObject)>, Vec<String>, i64), Error>;

    /// Return calendar-query results on a calendar view
    /// Since the <calendar-query> rules are rather complex this function
    /// is only meant to do some prefiltering
    async fn calendar_query(
        &self,
        principal: &str,
        name: &str,
        _query: CalendarQuery,
    ) -> Result<Vec<(String, CalendarObject)>, Error> {
        self.get_objects(principal, name).await
    }

    /// List calendar objects for a calendar view
    async fn get_objects(
        &self,
        principal: &str,
        name: &str,
    ) -> Result<Vec<(String, CalendarObject)>, Error>;

    /// Get a calendar object from a calendar view
    async fn get_object(
        &self,
        principal: &str,
        name: &str,
        object_id: &str,
        show_deleted: bool,
    ) -> Result<CalendarObject, Error>;

    async fn put_objects(
        &self,
        principal: &str,
        cal_id: &str,
        objects: Vec<(String, CalendarObject)>,
        overwrite: bool,
    ) -> Result<(), Error>;

    async fn put_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        object: CalendarObject,
        overwrite: bool,
    ) -> Result<(), Error> {
        self.put_objects(
            principal,
            cal_id,
            vec![(object_id.to_owned(), object)],
            overwrite,
        )
        .await
    }

    async fn delete_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
        use_trashbin: bool,
    ) -> Result<(), Error>;

    async fn restore_object(
        &self,
        principal: &str,
        cal_id: &str,
        object_id: &str,
    ) -> Result<(), Error>;

    async fn calendar_metadata(
        &self,
        principal: &str,
        cal_id: &str,
    ) -> Result<CollectionMetadata, Error>;

    // read_only refers to objects, metadata may still be updated
    fn is_read_only(&self, cal_id: &str) -> bool;
}
