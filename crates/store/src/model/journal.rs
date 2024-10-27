use ical::parser::ical::component::IcalJournal;

#[derive(Debug, Clone)]
pub struct JournalObject {
    pub(crate) journal: IcalJournal,
}
