use derive_more::From;
use ical::parser::ical::component::IcalJournal;

#[derive(Debug, Clone, From)]
pub struct JournalObject(pub IcalJournal);

impl JournalObject {
    pub fn get_uid(&self) -> &str {
        self.0.get_uid()
    }
}
