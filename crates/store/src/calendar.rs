use crate::synctoken::format_synctoken;
use chrono::NaiveDateTime;
use rustical_ical::CalendarObjectType;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct BaseCalendar {
    // A globally unique identifier. This identifier is not user-facing since users will only
    // access calendar views
    pub id: Uuid,
    // The calendar owner. Only the owner should be allowed to manage views/shares.
    // If the owner deletes their view the whole calendar is deleted.
    pub owner: String,
    pub timezone_id: Option<String>,
    // Tracks the version of calendar object updates. Equal for all sharees.
    pub synctoken: i64,
    // Used in case the calendar is a subscription
    pub subscription_url: Option<String>,
    // Push topic specified by WebDAV Push; used to send push notifications about this calendar.
    // Since only object changes are published, this is equal for all sharees.
    pub push_topic: String,
    // Object components to accept: VEVENT,VTODO,VJOURNAL
    pub components: Vec<CalendarObjectType>,
}

pub trait BaseCal: std::fmt::Debug + Clone + PartialEq + Eq {}

impl BaseCal for BaseCalendar {}
impl BaseCal for Uuid {}

/// A calendar view.
///
/// The `cal` field can either be a
/// - `uuid::Uuid` to specify the base calendar by reference (`CalendarView`)
/// - `BaseCalendar` to specify the base calendar by value (`FullCalendarView`)
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct CalendarView<CAL: BaseCal = Uuid> {
    // Either the BaseCalendar or a reference
    pub cal: CAL,
    // The principal that sees this calendar view
    pub principal: String,
    // The name this view appears as:
    // `/caldav/principal/<principal>/<name>/`
    pub name: String,
    // When the calendar view has been moved to the trash bin.
    // Can also be used to reject a share.
    pub deleted_at: Option<NaiveDateTime>,

    // The displayname can be set by each sharee individually.
    pub displayname: String,
    // The description can be set by each sharee individually.
    pub description: Option<String>,
    // The order can be set by each sharee individually.
    pub order: i64,
    // The color can be set by each sharee individually.
    pub color: Option<String>,
}

pub type FullCalendarView = CalendarView<BaseCalendar>;

impl From<FullCalendarView> for CalendarView {
    fn from(value: FullCalendarView) -> Self {
        Self {
            cal: value.cal.id,
            principal: value.principal,
            name: value.name,
            deleted_at: value.deleted_at,
            displayname: value.displayname,
            description: value.description,
            order: value.order,
            color: value.color,
        }
    }
}

impl FullCalendarView {
    /// A view is a share when the principal of the view is not the calendar owner.
    fn is_share(&self) -> bool {
        self.cal.owner != self.principal
    }
}

impl BaseCalendar {
    #[must_use]
    pub fn format_synctoken(&self) -> String {
        format_synctoken(self.synctoken)
    }

    #[must_use]
    pub fn get_timezone(&self) -> Option<chrono_tz::Tz> {
        self.timezone_id
            .as_ref()
            .and_then(|tzid| chrono_tz::Tz::from_str(tzid).ok())
    }

    #[must_use]
    pub fn get_vtimezone(&self) -> Option<&'static str> {
        self.timezone_id
            .as_ref()
            .and_then(|tzid| vtimezones_rs::VTIMEZONES.get(tzid).copied())
    }
}
