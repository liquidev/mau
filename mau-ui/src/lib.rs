pub extern crate paws;

mod input;
mod render;

use std::ops::{Deref, DerefMut};

pub use input::*;
pub use render::*;

/// paws UI state specialized to the selected backend, and extended with input capabilities.
pub struct Ui {
    /// For convenience, this field is also accessible via `Deref`.
    pub ui: paws::Ui<Backend>,
    pub input: Input,
}

impl Ui {
    /// Creates a new instance of the UI state.
    pub fn new(renderer: Backend) -> Self {
        Self {
            ui: paws::Ui::new(renderer),
            input: Input::new(),
        }
    }
}

impl Deref for Ui {
    type Target = paws::Ui<Backend>;

    fn deref(&self) -> &Self::Target {
        &self.ui
    }
}

impl DerefMut for Ui {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ui
    }
}
