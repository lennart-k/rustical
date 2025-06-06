use ical::parser::ical::component::IcalTodo;

#[derive(Debug, Clone)]
pub struct TodoObject {
    pub todo: IcalTodo,
    pub(crate) ics: String,
}
