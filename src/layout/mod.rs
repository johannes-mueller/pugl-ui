//! Facilities for widget layouting
//!
//! So far there is the classical box
//! stacking layout (like Gtk's HBox/Vbox) implemented. Other
//! layouting algorithms can be implemented later.
//!
//! This module contains the items, that are needed to layout
//! widgets.
//!
//! # Principles
//!
//! The [`UI`](../ui/struct.UI.html) knows the widget tree as a tree
//! of [`WidgetNode`](../ui/struct.WidgetNode.html)s. Each node of the
//! widget tree has an associated
//! [`LayouterImpl`](trait.LayouterImpl.html) trait object, that is
//! responsible of layouting the children.
//!
//! The layouting process has two stages.
//!
//! * Size determination
//!
//!   In the first stage the `UI` crawls recrusively through the
//!   widget tree and asks all the widget for their minimal sizes this
//!   happens recursively as the layouting widgets ask their children
//!   about their minimal size and and then calculate how the size of
//!   the layout would be.  A [`LayouterImpl`](trait.LayouterImpl.html)
//!   trait object must therefor implement
//!   [`LayouterImpl::calc_widget_sizes()`](trait.LayouterImpl.html#tymethod.calc_widget_sizes).
//!
//! * Size application
//!
//!   Once the minimal size of each widget and each (sub)layout is
//!   known, the sizes are applied to the individual widgets. A
//!   layouter can choose to expand widgets to fit the layout better,
//!   if the widget signals expandability.  During size application
//!   the layouter also sets the position of the widget.
//!   All this happens in [`LayouterImpl::apply_sizes()`](trait.LayouterImpl.html#tymethod.apply_sizes).
//!
use downcast_rs::DowncastSync;

use pugl_sys as sys;
use crate::ui;
use crate::widget;

pub mod stacklayout;

#[doc(hidden)]
pub mod layoutwidget;

#[doc(inline)]
pub use layoutwidget::*;

/// A trait describing layouters in order to assign them to a
/// [`LayoutWidget`](struct.LayoutWidget.html).
///
///
pub trait Layouter : Default + Clone + Copy {
    /// A type to describe the target where the Layouter is supposed to put the widget.
    type Target;
    /// The actual layout performing type
    type Implementor: LayouterImpl;
    fn new_implementor() -> Box<dyn LayouterImpl>;
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: widget::Id, target: Self::Target);
    fn expandable() -> (bool, bool);
}

pub trait LayouterImpl: DowncastSync {
    fn apply_sizes(
        &self,
        widgets: &mut Vec<Box<dyn widget::Widget>>,
        children: &[ui::WidgetNode],
        orig_pos: sys::Coord,
        available_size: sys::Size);
    fn calc_widget_sizes(
        &self,
        widgets: &mut Vec<Box<dyn widget::Widget>>,
        children: &[ui::WidgetNode]) -> sys::Size;
}
impl_downcast!(sync LayouterImpl);
