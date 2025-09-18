use derive_more::From;
use ical::parser::ical::component::IcalTodo;

#[derive(Debug, Clone, From)]
pub struct TodoObject(pub IcalTodo);

impl TodoObject {
    pub fn get_uid(&self) -> &str {
        self.0.get_uid()
    }
}
