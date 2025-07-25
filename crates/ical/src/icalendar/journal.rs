use derive_more::From;
use ical::parser::ical::component::IcalJournal;

#[derive(Debug, Clone, From)]
pub struct JournalObject(pub IcalJournal);
