use ical::parser::ical::component::IcalTodo;

#[derive(Debug, Clone)]
pub struct TodoObject {
    pub(crate) todo: IcalTodo,
}
