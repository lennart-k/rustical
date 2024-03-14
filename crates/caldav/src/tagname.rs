use strum::EnumProperty;

pub trait TagName {
    fn tagname(self) -> &'static str;
}

impl<P: EnumProperty + Into<&'static str>> TagName for P {
    fn tagname(self) -> &'static str {
        self.get_str("tagname").unwrap_or(self.into())
    }
}
