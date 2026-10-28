//! How a calendar object looks to someone the calendar is shared with.
//!
//! The owner decides per CLASS how much a sharee sees ([`Policy`]); the strictest CLASS of all
//! components of an object decides for the whole object ([`level_of`]). Below [`FULL`] a view
//! keeps only an allowlist of properties ([`view`]), so nothing the list does not name can leak.
use crate::CalendarObject;
use caldata::ContentLineParser;
use caldata::generator::Emitter;
use caldata::parser::{ContentLine, ContentLineParams};
use std::collections::{BTreeMap, HashSet};
use tracing::warn;

/// The object does not exist for the sharee
pub const HIDDEN: u8 = 0;
/// Time, status and transparency only
pub const BUSY: u8 = 1;
/// [`BUSY`] and the summary
pub const TITLE: u8 = 2;
/// [`TITLE`] and the location
pub const TITLE_LOCATION: u8 = 3;
/// Everything except alarms
pub const FULL: u8 = 4;

/// How objects of a calendar with CLASS:PRIVATE and CLASS:CONFIDENTIAL appear to its sharees
/// (hidden unless its owner sets otherwise). `confidential` is never above `private`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Policy {
    pub private: u8,
    pub confidential: u8,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            private: HIDDEN,
            confidential: HIDDEN,
        }
    }
}

impl Policy {
    /// A policy if the levels are valid: both at most [`FULL`], CONFIDENTIAL not above PRIVATE
    #[must_use]
    pub const fn new(private: u8, confidential: u8) -> Option<Self> {
        if private > FULL || confidential > private {
            return None;
        }
        Some(Self {
            private,
            confidential,
        })
    }
}

const MAIN_COMPONENTS: [&str; 3] = ["VEVENT", "VTODO", "VJOURNAL"];

/// Properties a view below [`FULL`] keeps on an event or a task, by the lowest level they appear at
const ALLOWED: &[(&str, u8)] = &[
    ("UID", BUSY),
    ("DTSTAMP", BUSY),
    ("CREATED", BUSY),
    ("LAST-MODIFIED", BUSY),
    ("SEQUENCE", BUSY),
    ("DTSTART", BUSY),
    ("DTEND", BUSY),
    ("DURATION", BUSY),
    ("DUE", BUSY),
    ("RRULE", BUSY),
    ("RDATE", BUSY),
    ("EXDATE", BUSY),
    ("EXRULE", BUSY),
    ("RECURRENCE-ID", BUSY),
    ("STATUS", BUSY),
    ("TRANSP", BUSY),
    ("CLASS", BUSY),
    ("SUMMARY", TITLE),
    ("LOCATION", TITLE_LOCATION),
    ("GEO", TITLE_LOCATION),
];

/// Parameters a view below [`FULL`] keeps
const ALLOWED_PARAMS: [&str; 3] = ["TZID", "VALUE", "RANGE"];

/// Properties of the calendar itself a view below [`FULL`] keeps
const ALLOWED_CALENDAR: [&str; 3] = ["VERSION", "PRODID", "CALSCALE"];

/// An iCalendar component as a tree of content lines, in the order of the source
#[derive(Debug, Clone)]
struct Component {
    name: String,
    items: Vec<Item>,
}

#[derive(Debug, Clone)]
enum Item {
    Property(ContentLine),
    Component(Component),
}

impl Component {
    fn parse(ics: &str) -> Option<Self> {
        let mut stack: Vec<Self> = vec![];
        let mut root = None;
        for line in ContentLineParser::from_slice(ics.as_bytes()) {
            let line = line.ok()?;
            match line.name.as_str() {
                "BEGIN" => stack.push(Self {
                    name: line.value.to_uppercase(),
                    items: vec![],
                }),
                "END" => {
                    let component = stack.pop()?;
                    if component.name != line.value.to_uppercase() {
                        return None;
                    }
                    match stack.last_mut() {
                        Some(parent) => parent.items.push(Item::Component(component)),
                        None if root.is_none() => root = Some(component),
                        None => return None,
                    }
                }
                _ => stack.last_mut()?.items.push(Item::Property(line)),
            }
        }
        if !stack.is_empty() {
            return None;
        }
        root.filter(|root| root.name == "VCALENDAR")
    }

    fn emit(&self, output: &mut String) {
        output.push_str("BEGIN:");
        output.push_str(&self.name);
        output.push_str("\r\n");
        for item in &self.items {
            match item {
                // Folded, with the line break
                Item::Property(line) => line
                    .generate(output)
                    .expect("Formatting to a string is infallible"),
                Item::Component(component) => component.emit(output),
            }
        }
        output.push_str("END:");
        output.push_str(&self.name);
        output.push_str("\r\n");
    }

    fn to_ics(&self) -> String {
        let mut output = String::new();
        self.emit(&mut output);
        output
    }

    fn properties<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a ContentLine> + 'a {
        self.items.iter().filter_map(move |item| match item {
            Item::Property(line) if line.name == name => Some(line),
            _ => None,
        })
    }

    fn main_components(&self) -> impl Iterator<Item = &Self> {
        self.items.iter().filter_map(|item| match item {
            Item::Component(component) if MAIN_COMPONENTS.contains(&component.name.as_str()) => {
                Some(component)
            }
            _ => None,
        })
    }

    fn main_components_mut(&mut self) -> impl Iterator<Item = &mut Self> {
        self.items.iter_mut().filter_map(|item| match item {
            Item::Component(component) if MAIN_COMPONENTS.contains(&component.name.as_str()) => {
                Some(component)
            }
            _ => None,
        })
    }

    fn recurrence_id(&self) -> Option<String> {
        self.properties("RECURRENCE-ID")
            .next()
            .map(|line| line.value.clone())
    }

    fn alarms(&self) -> Vec<Item> {
        self.items
            .iter()
            .filter(|item| matches!(item, Item::Component(component) if component.name == "VALARM"))
            .cloned()
            .collect()
    }

    /// Removes every VALARM anywhere below this component
    fn remove_alarms(&mut self) {
        self.items.retain(
            |item| !matches!(item, Item::Component(component) if component.name == "VALARM"),
        );
        for item in &mut self.items {
            if let Item::Component(component) = item {
                component.remove_alarms();
            }
        }
    }
}

/// What the level of an object depends on: the CLASS values of its main components.
///
/// Built once per object, it gives the level under any [`Policy`] without reading the object
/// again ([`Self::level`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Classes {
    /// The object cannot be read: [`HIDDEN`] under every policy
    Unreadable,
    Readable {
        /// Some main component has CLASS:CONFIDENTIAL
        confidential: bool,
        /// Some main component has a CLASS other than PUBLIC and CONFIDENTIAL
        private: bool,
        journal: bool,
    },
}

impl Classes {
    /// The classes of the iCalendar text `ics`
    #[must_use]
    pub fn of(ics: &str) -> Self {
        Component::parse(ics).map_or(Self::Unreadable, |calendar| Self::of_calendar(&calendar))
    }

    fn of_calendar(calendar: &Component) -> Self {
        let (mut confidential, mut private, mut journal) = (false, false, false);
        for component in calendar.main_components() {
            journal |= component.name == "VJOURNAL";
            for line in component.properties("CLASS") {
                // RFC 5545 section 3.8.1.3: case-insensitive, an unknown class is treated as PRIVATE
                match line.value.trim().to_uppercase().as_str() {
                    "PUBLIC" => {}
                    "CONFIDENTIAL" => confidential = true,
                    _ => private = true,
                }
            }
        }
        Self::Readable {
            confidential,
            private,
            journal,
        }
    }

    /// Whether the object can be read at all
    #[must_use]
    pub const fn readable(self) -> bool {
        matches!(self, Self::Readable { .. })
    }

    /// The level at which the object appears to sharees under `policy`.
    ///
    /// The strictest CLASS of all its components decides (one confidential occurrence of a
    /// series makes the whole series confidential).
    #[must_use]
    pub fn level(self, policy: Policy) -> u8 {
        let Self::Readable {
            confidential,
            private,
            journal,
        } = self
        else {
            return HIDDEN;
        };
        let mut level = FULL;
        if confidential {
            level = level.min(policy.confidential);
        }
        if private {
            level = level.min(policy.private);
        }
        // Journals are text: there is nothing to show of them below FULL
        if journal && level < FULL {
            HIDDEN
        } else {
            level
        }
    }
}

/// Whether the level of the iCalendar text `ics` may depend on the policy, without parsing it.
///
/// `false` only if no line is a CLASS other than PUBLIC: then the object has the same level under
/// every policy ([`FULL`], or [`HIDDEN`] if it cannot be read). It unfolds lines as the parser
/// does and answers `true` whenever it is unsure.
#[must_use]
pub fn may_depend_on_policy(ics: &str) -> bool {
    let mut lines = ics
        .split('\n')
        .map(|part| part.strip_suffix('\r').unwrap_or(part))
        .peekable();
    let mut line = String::new();
    while let Some(start) = lines.next() {
        line.clear();
        line.push_str(start);
        // A folded line continues on lines that start with a space or a tab
        while let Some(continuation) = lines.peek().and_then(|part| part.strip_prefix([' ', '\t']))
        {
            line.push_str(continuation);
            lines.next();
        }
        if !line
            .get(..5)
            .is_some_and(|name| name.eq_ignore_ascii_case("CLASS"))
        {
            continue;
        }
        let rest = &line[5..];
        if let Some(value) = rest.strip_prefix(':') {
            if !value.trim().eq_ignore_ascii_case("PUBLIC") {
                return true;
            }
        } else if rest.starts_with(';') {
            // Parameters: left to the parser
            return true;
        }
    }
    false
}

/// The level at which `object` appears to sharees under `policy` ([`Classes::level`]).
/// An object that cannot be read is [`HIDDEN`].
#[must_use]
pub fn level_of(object: &CalendarObject, policy: Policy) -> u8 {
    let classes = Classes::of(object.get_ics());
    if !classes.readable() {
        warn!(
            uid = object.get_uid(),
            "Calendar object cannot be read for its sharees, hidden from them"
        );
    }
    classes.level(policy)
}

fn restricted_component(component: &Component, level: u8) -> Component {
    let items = component
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Property(line) => {
                let (_, min_level) = ALLOWED.iter().find(|(name, _)| *name == line.name)?;
                if *min_level > level {
                    return None;
                }
                let params: Vec<(String, Vec<String>)> = ALLOWED_PARAMS
                    .iter()
                    .filter_map(|&param| {
                        line.params
                            .get_param(param)
                            .map(|value| (param.to_owned(), vec![value.to_owned()]))
                    })
                    .collect();
                Some(Item::Property(ContentLine {
                    name: line.name.clone(),
                    params: ContentLineParams::from(params),
                    value: line.value.clone(),
                }))
            }
            // Alarms, locations, participants, ...: none below FULL
            Item::Component(_) => None,
        })
        .collect();
    Component {
        name: component.name.clone(),
        items,
    }
}

fn restricted_calendar(calendar: &Component, level: u8) -> Component {
    let mut items = vec![];
    let mut components = vec![];
    let mut tzids = HashSet::new();
    for item in &calendar.items {
        match item {
            Item::Property(line) if ALLOWED_CALENDAR.contains(&line.name.as_str()) => {
                items.push(Item::Property(ContentLine {
                    name: line.name.clone(),
                    params: ContentLineParams::default(),
                    value: line.value.clone(),
                }));
            }
            Item::Component(component) if MAIN_COMPONENTS.contains(&component.name.as_str()) => {
                let restricted = restricted_component(component, level);
                for item in &restricted.items {
                    if let Item::Property(line) = item
                        && let Some(tzid) = line.params.get_tzid()
                    {
                        tzids.insert(tzid.to_owned());
                    }
                }
                components.push(Item::Component(restricted));
            }
            _ => {}
        }
    }
    // The time zones the kept properties refer to
    for item in &calendar.items {
        if let Item::Component(component) = item
            && component.name == "VTIMEZONE"
            && component
                .properties("TZID")
                .any(|line| tzids.contains(&line.value))
        {
            items.push(item.clone());
        }
    }
    items.extend(components);
    Component {
        name: calendar.name.clone(),
        items,
    }
}

/// `object` as a sharee sees it at `level`; `None` if the sharee does not see it at all
/// (also if no view can be built: never an error, never the raw data).
#[must_use]
pub fn view(object: &CalendarObject, level: u8) -> Option<CalendarObject> {
    if level == HIDDEN {
        return None;
    }
    let Some(mut calendar) = Component::parse(object.get_ics()) else {
        warn!(
            uid = object.get_uid(),
            "Calendar object cannot be read for its sharees, hidden from them"
        );
        return None;
    };
    let view = if level >= FULL {
        calendar.remove_alarms();
        calendar
    } else {
        if calendar
            .main_components()
            .any(|component| component.name == "VJOURNAL")
        {
            return None;
        }
        restricted_calendar(&calendar, level)
    };
    let view = CalendarObject::from_ics(view.to_ics());
    if view.is_err() {
        warn!(
            uid = object.get_uid(),
            level, "No view of a calendar object can be built for its sharees, hidden from them"
        );
    }
    view.ok()
}

/// [`view`] at the level `policy` gives the object
#[must_use]
pub fn view_for(object: &CalendarObject, policy: Policy) -> Option<CalendarObject> {
    view(object, level_of(object, policy))
}

/// What a sharee with write access sends, with the alarms of what is stored.
///
/// Every component keeps the alarms of the stored component with the same RECURRENCE-ID, a new
/// occurrence those of the stored master; alarms the sharee sent are dropped. With nothing
/// stored the result has no alarms. `None` if either cannot be read.
#[must_use]
pub fn with_stored_alarms(
    sent: &CalendarObject,
    stored: Option<&CalendarObject>,
) -> Option<CalendarObject> {
    let mut sent = Component::parse(sent.get_ics())?;
    let stored_alarms: BTreeMap<Option<String>, Vec<Item>> = match stored {
        Some(stored) => Component::parse(stored.get_ics())?
            .main_components()
            .map(|component| (component.recurrence_id(), component.alarms()))
            .collect(),
        None => BTreeMap::new(),
    };
    let master_alarms = stored_alarms.get(&None).cloned().unwrap_or_default();
    for component in sent.main_components_mut() {
        component.remove_alarms();
        let alarms = stored_alarms
            .get(&component.recurrence_id())
            .cloned()
            .unwrap_or_else(|| master_alarms.clone());
        component.items.extend(alarms);
    }
    CalendarObject::from_ics(sent.to_ics()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn ics(lines: &str) -> String {
        format!(
            "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//test//EN\r\nBEGIN:VEVENT\r\nUID:u\r\n\
             DTSTAMP:20260101T000000Z\r\nDTSTART:20260105T100000Z\r\n{lines}END:VEVENT\r\nEND:VCALENDAR\r\n"
        )
    }

    #[rstest]
    #[case("", false)]
    #[case("CLASS:PUBLIC\r\n", false)]
    #[case("class:public \r\n", false)]
    #[case("SUMMARY:CLASS:PRIVATE\r\n", false)]
    #[case("X-CLASS:PRIVATE\r\nCLASSES:PRIVATE\r\n", false)]
    #[case("CLASS:PRIVATE\r\n", true)]
    #[case("Class:Confidential\r\n", true)]
    #[case("CLASS:X-SECRET\r\n", true)]
    #[case("CLASS;X-P=1:PUBLIC\r\n", true)]
    // Folded anywhere, also within the name
    #[case("CLA\r\n SS:PRIVATE\r\n", true)]
    #[case("CLASS:PUB\r\n\tLIC\r\n", false)]
    #[case("CLASS:PUBLIC\r\n X\r\n", true)]
    fn test_prefilter(#[case] lines: &str, #[case] depends: bool) {
        assert_eq!(may_depend_on_policy(&ics(lines)), depends, "{lines:?}");
    }

    /// Whatever the prefilter lets through unparsed has one level under every policy
    #[rstest]
    #[case("")]
    #[case("CLASS:PUBLIC\r\n")]
    #[case("SUMMARY:CLASS:PRIVATE\r\n")]
    fn test_prefilter_skips_only_policy_independent(#[case] lines: &str) {
        let ics = ics(lines);
        assert!(!may_depend_on_policy(&ics));
        let classes = Classes::of(&ics);
        for private in HIDDEN..=FULL {
            for confidential in HIDDEN..=private {
                let policy = Policy::new(private, confidential).unwrap();
                assert_eq!(classes.level(policy), FULL);
            }
        }
        // Text that cannot be read: hidden under every policy
        let broken = Classes::of(&ics.replace("END:VEVENT", "END:VTODO"));
        assert!(!broken.readable());
        assert_eq!(broken.level(Policy::new(FULL, FULL).unwrap()), HIDDEN);
    }

    #[test]
    fn test_classes_levels() {
        let policy = Policy::new(3, 1).unwrap();
        assert_eq!(Classes::of(&ics("CLASS:PRIVATE\r\n")).level(policy), 3);
        assert_eq!(Classes::of(&ics("CLASS:CONFIDENTIAL\r\n")).level(policy), 1);
        assert_eq!(Classes::of(&ics("CLASS:X-OTHER\r\n")).level(policy), 3);
        assert_eq!(Classes::of(&ics("CLASS:PUBLIC\r\n")).level(policy), FULL);
    }
}
