//! A module for widget layouting. So far there is the classical box
//! stacking layout (like Gtk's HBox/Vbox) implemented. Other
//! layouting algorithms can be implemented later.
//!
//! This module contains the items, that are needed to layout
//! widgets.
//!
//! # Usage
//!

use crate::widget::*;

pub type Spacing = f64;

pub enum StackDirection {
    Front,
    Back
}

#[derive(Clone, Copy, Default, Debug)]
pub struct HorizontalLayouter;

#[derive(Clone, Copy, Default, Debug)]
pub struct VerticalLayouter;

#[derive(Default)]
pub struct Spacer {
    stub: WidgetStub,
    width_expandable: bool,
    height_expandable: bool
}

impl Widget for Spacer {
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
    fn width_expandable(&self) -> bool { self.width_expandable }
    fn height_expandable(&self) -> bool { self.height_expandable }
}

impl Spacer {
    pub(crate) fn set_expandable(&mut self, (we, he): (bool, bool)) {
        self.width_expandable = we;
        self.height_expandable = he;
    }
}
