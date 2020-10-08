//! The widget that contains a layout

use crate::layout;
use crate::widget;

/// The widget that contains a layout
///
/// In the [`WidgetNode`](../ui/struct.WidgetNode.html) tree the nodes
/// have child widgets that need to be layouted by a
/// [`Layouter`](trait.Layouter.html). Usually at the nodes there is a
/// `LayoutWidget` that has functionality to support the underlying
/// `Layouter`.
#[derive(Default)]
pub struct LayoutWidget {
    stub: widget::WidgetStub,
    width_expandable: bool,
    height_expandable: bool,

    width_locked: bool,
    height_locked: bool,
}

impl LayoutWidget {
    pub(crate) fn set_expandable(&mut self, we: bool, he: bool) {
        self.width_expandable = we && !self.width_locked;
        self.height_expandable = he && !self.height_locked;
    }

    /// Locks the width of the widget.
    ///
    /// If the width of the widget is *not* locked, the widget can be
    /// set to be [`width_expandlable()`](../widget/trait.Widget.html#method.width_expandable).
    /// That means if the width of widget *is* locked, the widget will
    /// *not* let itself expanded, even if all the children would
    /// permit their width to be expanded.
    pub fn lock_width(&mut self) {
        self.width_locked = true;
    }
    /// Locks the height of the widget.
    ///
    /// If the height of the widget is *not* locked, the widget can be
    /// set to be [`height_expandlable()`](../widget/trait.Widget.html#method.height_expandable).
    /// That means if the height of widget *is* locked, the widget will
    /// *not* let itself expanded, even if all the children would
    /// permit their height to be expanded.
    pub fn lock_height(&mut self) {
        self.height_locked = true;
    }
}

impl widget::Widget for LayoutWidget {
    fn stub (&self) -> &widget::WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut widget::WidgetStub {
        &mut self.stub
    }

    fn width_expandable(&self) -> bool { self.width_expandable }
    fn height_expandable(&self) -> bool { self.height_expandable }

    fn sized_width(&self) -> bool { true }
    fn sized_height(&self) -> bool { true }
    fn pointer_enter_wrap(&mut self) {}
    fn pointer_leave_wrap(&mut self) {}
}

/// A handle that contains a [`WidgetHandle`](../widget/WidgetHandle.html).
pub struct LayoutWidgetHandle<L: layout::Layouter, W: widget::Widget> {
    widget_handle: widget::WidgetHandle<W>,
    layouter: L
}

impl<L: layout::Layouter, W: widget::Widget> Clone for LayoutWidgetHandle<L, W> {
    fn clone(&self) -> Self {
        LayoutWidgetHandle::<L, W> {
            widget_handle: self.widget_handle,
            layouter: L::default()
        }
    }
}

impl<L: layout::Layouter, W: widget::Widget> Copy for LayoutWidgetHandle<L, W> { }

impl<L: layout::Layouter, W: widget::Widget> LayoutWidgetHandle<L, W> {
    pub fn new(widget_handle: widget::WidgetHandle<W>) -> LayoutWidgetHandle<L, W> {
        LayoutWidgetHandle { widget_handle, layouter: L::default() }
    }
    pub fn widget(&self) -> widget::WidgetHandle<W> {
        self.widget_handle
    }

    pub fn layouter(&mut self) -> &mut L {
        &mut self.layouter
    }
    pub fn expandable() -> (bool, bool) {
        L::expandable()
    }
}
