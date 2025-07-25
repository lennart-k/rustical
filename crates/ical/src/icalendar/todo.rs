use derive_more::From;
use ical::parser::ical::component::IcalTodo;

#[derive(Debug, Clone, From)]
pub struct TodoObject(pub IcalTodo);
