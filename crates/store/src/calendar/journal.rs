use ical::parser::ical::component::IcalJournal;

#[derive(Debug, Clone)]
pub struct JournalObject {
    pub journal: IcalJournal,
}
