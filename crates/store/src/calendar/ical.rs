pub trait IcalProperty {
    fn get_param(&self, name: &str) -> Option<Vec<&str>>;
    fn get_value_type(&self) -> Option<&str>;
    fn get_tzid(&self) -> Option<&str>;
}

impl IcalProperty for ical::property::Property {
    fn get_param(&self, name: &str) -> Option<Vec<&str>> {
        self.params
            .as_ref()?
            .iter()
            .find(|(key, _)| name == key)
            .map(|(_, value)| value.iter().map(String::as_str).collect())
    }

    fn get_value_type(&self) -> Option<&str> {
        self.get_param("VALUE")
            .and_then(|params| params.into_iter().next())
    }

    fn get_tzid(&self) -> Option<&str> {
        self.get_param("TZID")
            .and_then(|params| params.into_iter().next())
    }
}
