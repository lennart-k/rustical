pub trait InvalidProperty {
    fn invalid_property(&self) -> bool;
}

impl<T: Default + PartialEq> InvalidProperty for T {
    fn invalid_property(&self) -> bool {
        self == &T::default()
    }
}
