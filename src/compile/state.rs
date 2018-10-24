use std::ops::{Deref, DerefMut};
use crate::compile::Driver;

pub struct State<'driver> {
    /// Driver for this compiler state.
    pub (in super) driver: &'driver mut Driver,
}

impl<'driver> Deref for State<'driver> {
    type Target = Driver;

    fn deref(&self) -> &Driver {
        &self.driver
    }
}

impl<'driver> DerefMut for State<'driver> {
    fn deref_mut(&mut self) -> &mut Driver {
        &mut self.driver
    }
}
