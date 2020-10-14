//! Everything to describe an access a widget
use std::marker::PhantomData;
use downcast_rs::DowncastSync;

use pugl_sys::*;

/// The unique Id of a widget.
///
/// The Id is the way, widgets can be accessed by a [`WidgetHandle`](struct.WidgetHandle.html).
pub type Id = usize;

/// The `Widget` trait.
///
/// Widgets need to implement this trait. Most of the methods have
/// default implementations, so that simple widgets can be easily
/// defined. Eeven layouts are internally treated as widgets.
///
/// Data common to all widgets is kept in the struct
/// [`WidgetStub`](struct.WidgetStub.html) accessible from the widget by
/// the methods [`stub()`](#tymethod.stub) and [`stub_mut()`](#tymethod.stub_mut).
pub trait Widget : DowncastSync {

    /// Called by the UI to pass an event to the widget.
    ///
    /// The widget is supposed to process the Event and return `None`
    /// if the widget has processed the event. If the widget has not
    /// processed the event it shoud return `Some(ev)` so that the
    /// event can be passed to its parent widget.
    ///
    /// There is [`EventState`](../ui/enum.EventState.html) and the macros
    /// [`event_processed!()`](../macro.event_processed.html) and
    /// [`event_not_processed!()`](../macro.event_not_processed.html) to do this.
    ///
    /// The default implementation just passes the event without touching it.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let ev = Event {
    ///     data: EventType::MouseButtonPress(MouseButton { num: 1, modifiers: 0 }),
    ///     context: Default::default()
    /// };
    /// let mut widget = DummyWidget::default();
    ///
    /// assert_eq!(widget.event(ev), Some(ev));
    /// # }
    /// ```
    fn event(&mut self, ev: Event) -> Option<Event> {
        Some (ev)
    }

    /// Called when the widget has to draw itself.
    ///
    /// # Parameters
    ///
    /// * `expose: &ExposeArea` – a pugl_sys::pugl::ExposeArea
    /// carrying the information which rectangle of the widget
    /// actually needs to be redrawn.
    ///
    /// Default implementation does nothing.
    fn exposed(&mut self, _expose: &ExposeArea, _cr: &cairo::Context) {}

    /// Supposed to return the minimum size of the widget.
    ///
    /// Default: zero size
    fn min_size(&self) -> Size { Default::default() }

    /// Suposed to return true iff the widget is expandable in x-direction
    ///
    /// Default: `false`
    fn width_expandable (&self) -> bool { false }

    /// Suposed to return true iff the widget is expandable in y-direction
    ///
    /// Default: `false`
    fn height_expandable (&self) -> bool { false }

    /// Supposed to return true iff the widget can take the focus.
    ///
    /// Default: `false`
    fn takes_focus (&self) -> bool {
        false
    }

    /// Called when the mouse pointer is entering the widget's layout.
    ///
    /// Default implementation does nothing.
    fn pointer_enter(&mut self) {}

    /// Called when the mouse pointer is leaving the widget's layout.
    ///
    /// Default implementation does nothing.
    fn pointer_leave(&mut self) {}

    /// Called when the requested reminding time is passed
    ///
    /// Supposed to return true, iff the reminder is still needed
    ///
    /// Default implementation does nothing and returns false.
    fn reminder_handler(&mut self) -> bool { false }

    /// Supposed to return a reference to the `WidgetStub` of the widget
    ///
    /// usually implemented by the macro [`widget_stub!()`](../macro.widget_stub.html).
    fn stub (&self) -> &WidgetStub;

    /// Supposed to return a mutable reference to the `WidgetStub` of the widget.
    ///
    /// Usually implemented by the macro [`widget_stub!()`](../macro.widget_stub.html).
    fn stub_mut (&mut self) -> &mut WidgetStub;

    fn ask_for_repaint(&mut self)  {
        self.stub_mut().needs_repaint = true;
    }

    /// The widget can request a reminder after `timeout`
    /// seconds. When the time has passed `reminder_handler() is
    /// called.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.request_reminder(5.0);
    /// assert_eq!(widget.reminder_request(), Some(5.0));
    /// # }
    /// ```
    fn request_reminder(&mut self, timeout: f64) {
        self.stub_mut().reminder_request = Some(timeout);
    }

    /// Hands the reminder request over to the UI
    ///
    /// Only to be called by the UI as it consumes the reminder request.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// assert_eq!(widget.reminder_request(), None);
    /// widget.request_reminder(5.0);
    /// assert_eq!(widget.reminder_request(), Some(5.0));
    /// assert_eq!(widget.reminder_request(), None);
    /// # }
    /// ```
    fn reminder_request(&mut self) -> Option<f64> {
        self.stub_mut().reminder_request.take()
    }

    /// Returns true iff the widget is currently focused.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// assert!(!widget.has_focus());
    /// widget.set_focus(true);
    /// assert!(widget.has_focus());
    /// widget.set_focus(false);
    /// assert!(!widget.has_focus());
    /// # }
    /// ```
    fn has_focus(&self) -> bool {
        self.stub().has_focus
    }

    /// Returns the size of the widget after layouting.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert_eq!(widget.size(), Size { w: 137., h: 93.});
    /// # }
    /// ```
    fn size(&self) -> Size {
        self.stub().layout.size
    }

    /// Returns the positon (upper left corner of the widget)
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert_eq!(widget.pos(), Coord { x: 23., y: 42. });
    /// # }
    /// ```
    fn pos(&self) -> Coord {
        self.stub().layout.pos
    }

    /// Returns the six scalar values to conveniently describe the widget's geometry
    /// (left, right, top, bottom, width, height)
    ///
    /// Usually not to be reimplemented
    ///
    /// Useful in implementations of `::exposed()` when used as
    /// `let (left, right, top, bottom, width, height) = self.geometry();`
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert_eq!(widget.geometry(), (23., 160., 42., 135., 137., 93.));
    /// # }
    /// ```
    fn geometry(&self) -> (f64, f64, f64, f64, f64, f64) {
        let (x, y, w, h) = self.rect();
        (x, x+w, y, y+h, w, h)
    }

    /// Returns four scalar values to conveniently describe the widget's rectangle
    /// (x, y, width, height)
    ///
    /// Usually not to be reimplemented
    ///
    /// Useful in implementations of `::exposed()` when used as
    /// `let (x, y, w, h) = self.rect();`
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert_eq!(widget.rect(), (23., 42., 137., 93.));
    /// # }
    /// ```
    fn rect(&self) -> (f64, f64, f64, f64) {
        let x = self.pos().x;
        let y = self.pos().y;
        let w = self.size().w;
        let h = self.size().h;
        (x, y, w, h)
    }

    /// Returns true iff the widget has a defined minimum width
    ///
    /// Usually not to be reimplemented.
    fn sized_width(&self) -> bool {
        self.min_size().w > 0.0
    }

    /// Returns true iff the widget has a defined minimum height
    ///
    /// Usually not to be reimplemented.
    fn sized_height(&self) -> bool {
        self.min_size().h > 0.0
    }

    /// Sets the actual width of the widget to `width`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_width(23.);
    /// assert_eq!(widget.size().w, 23.);
    /// # }
    /// ```
    fn set_width (&mut self, width: f64) {
        self.stub_mut().layout.size.w = width;
    }

    /// Sets the actual height of the widget to `height`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_height(23.);
    /// assert_eq!(widget.size().h, 23.);
    /// # }
    /// ```
    fn set_height (&mut self, height: f64) {
        self.stub_mut().layout.size.h = height;
    }

    /// Expands the width of the widget by `amount`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_width(23.);
    /// widget.expand_width(42.);
    /// assert_eq!(widget.size().w, 65.);
    /// # }
    /// ```
    fn expand_width (&mut self, amount: f64) {
        self.stub_mut().layout.size.w += amount;
    }

    /// Expands the width of the widget by `amount`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_height(23.);
    /// widget.expand_height(42.);
    /// assert_eq!(widget.size().h, 65.);
    /// # }
    /// ```
    fn expand_height (&mut self, amount: f64) {
        self.stub_mut().layout.size.h += amount;
    }

    /// Sets the position of the widget to `pos`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_pos(&Coord { x: 23., y: 42. });
    /// assert_eq!(widget.pos(), Coord { x: 23., y: 42. });
    /// # }
    /// ```
    fn set_pos (&mut self, pos: &Coord) {
        self.stub_mut().layout.pos = *pos;
    }


    /// Sets the position of the widget to `size`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// widget.set_size(&Size { w: 23., h: 42. });
    /// assert_eq!(widget.size(), Size { w: 23., h: 42. });
    /// # }
    /// ```
    fn set_size (&mut self, size: &Size) {
        self.stub_mut().layout.size = *size;
    }

    /// Returns the [Layout](struct.Layout.html) (drawing rect) of the widget.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 0., y: 0. },
    ///     size: Size { w: 0., h: 0. }
    /// };
     /// assert_eq!(widget.layout(), layout);
    /// # }
    /// ```
    fn layout(&self) -> Layout {
        self.stub().layout
    }

    /// sets the Layout of the widget to `layout`
    ///
    /// Probably not needed as only used once in UI
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert_eq!(widget.layout(), layout);
    /// # }
    /// ```
    fn set_layout(&mut self, layout: &Layout) {
        self.stub_mut().layout = *layout;
    }

    /// Returns true iff the widget is sensitive to user evnets.
    ///
    /// Usually not to be reimplemented.
    fn is_sensitive(&self) -> bool {
        self.stub().sensitive
    }

    /// Returns true iff the widget is currently hovered.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// assert!(!widget.is_hovered());
    /// widget.pointer_enter_wrap();
    /// assert!(widget.is_hovered());
    /// widget.pointer_leave_wrap();
    /// assert!(!widget.is_hovered());
    /// # }
    /// ```
    fn is_hovered(&self) -> bool {
        self.stub().hovered
    }

    /// Returns true iff the widget's Layout is containing `pos`.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert!(widget.is_hit_by(Coord { x: 25., y: 45. }));
    /// assert!(!widget.is_hit_by(Coord { x: 15., y: 45. }));
    /// assert!(!widget.is_hit_by(Coord { x: 163., y: 45. }));
    /// assert!(!widget.is_hit_by(Coord { x: 25., y: 35. }));
    /// assert!(!widget.is_hit_by(Coord { x: 25., y: 137. }));
    /// # }
    /// ```
    fn is_hit_by (&self, pos: Coord) -> bool {
        let layout = self.stub().layout;

        let x1 = layout.pos.x;
        let x2 = x1 + layout.size.w;
        let y1 = layout.pos.y;
        let y2 = y1 + layout.size.h;
        (pos.x > x1 && pos.x < x2) && (pos.y > y1 && pos.y < y2)
    }

    /// Returns true iff the widget's Layout is intersecting `pos`.
    ///
    /// Usually not to be reimplemented.
    /// ```
    /// # use pugl_sys::*;
    /// # #[macro_use] extern crate pugl_ui;
    /// # use pugl_ui::widget::*;
    /// # #[derive(Default)] struct DummyWidget { stub: WidgetStub }
    /// # impl Widget for DummyWidget { widget_stub!(); }
    /// # fn main() {
    /// let mut widget = DummyWidget::default();
    /// let layout = Layout {
    ///     pos: Coord { x: 23., y: 42. },
    ///     size: Size { w: 137., h: 93. }
    /// };
    /// widget.set_layout(&layout);
    /// assert!(widget.intersects_with(
    ///     Coord { x: 12., y: 23. },
    ///     Size { w: 23., h: 23. }
    /// ));
    /// assert!(widget.intersects_with(
    ///     Coord { x: 150., y: 120. },
    ///     Size { w: 23., h: 23. }
    /// ));
    /// assert!(!widget.intersects_with(
    ///     Coord { x: 162., y: 132. },
    ///     Size { w: 23., h: 23. }
    /// ));
    /// assert!(!widget.intersects_with(
    ///     Coord { x: 12., y: 23. },
    ///     Size { w: 3., h: 3. }
    /// ));
    /// assert!(widget.intersects_with(
    ///     Coord { x: 12., y: 23. },
    ///     Size { w: 163., h: 143. }
    /// ));
    /// # }
    /// ```
    fn intersects_with(&self, pos: Coord, size: Size) -> bool {
        let layout = self.layout();

        let left = layout.pos.x;
        let right = left + layout.size.w;
        let a_left = pos.x;
        let a_right = pos.x + size.w;

        if left > a_right || right < a_left {
            return false;
        }

        let top = layout.pos.y;
        let bottom = top + layout.size.h;
        let a_top = pos.y;
        let a_bottom = pos.y + size.h;

        if top > a_bottom || bottom < a_top {
            return false;
        }

        true
    }

    /// Sets the widget's focus state to `yn`.
    ///
    /// Usually not to be reimplemented.
    fn set_focus(&mut self, yn: bool) {
        let hf = self.stub().has_focus;
        self.stub_mut().has_focus = yn;
        if hf != yn {
            self.stub_mut().needs_repaint = true;
        }
    }

    /// Returns true iff the widget needs to be repainted.
    ///
    /// Usually not to be reimplemented.
    fn needs_repaint(&mut self) -> bool {
        self.stub_mut().needs_repaint()
    }

    /// Wrapper for the `pointer_enter()` event function.
    ///
    /// Usually only called by the UI.
    /// Usually not to be reimplemented.
    fn pointer_enter_wrap(&mut self) {
        self.stub_mut().hovered = true;
        self.ask_for_repaint();
        self.pointer_enter();
    }

    /// Wrapper for the `pointer_leave()` event function.
    ///
    /// Usually only called by the UI.
    /// Usually not to be reimplemented.
    fn pointer_leave_wrap(&mut self) {
        self.stub_mut().hovered = false;
        self.ask_for_repaint();
        self.pointer_leave();
    }
}
impl_downcast!(sync Widget);

/// The rectangle the widget is covering
#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub struct Layout {
    pub pos: Coord,
    pub size: Size
}

/// The stub of a widget.
///
/// Contains all the data common to all widgets.
pub struct WidgetStub {
    pub layout: Layout,
    has_focus: bool,
    needs_repaint: bool,
    sensitive: bool,
    hovered: bool,
    reminder_request: Option<f64>
}

impl Default for WidgetStub {
    fn default() -> WidgetStub {
        WidgetStub {
            layout: Layout::default(),
            has_focus: false,
            needs_repaint: false,
            sensitive: true,
            hovered: false,
            reminder_request: None
        }
    }
}

impl WidgetStub {
    fn needs_repaint(&mut self) -> bool {
        let nrp = self.needs_repaint;
        self.needs_repaint = false;
        nrp
    }
}

/// A handle of a widget.
///
/// Contains the Id of the widget and a `PhantomData` of its actual
/// type.  When a widget of type `<T: Widget>` is registered in the
/// `UI` by [`UI::new_widget()`](../ui/struct.UI.html#method.new_widget)
/// an object `WidgetHandle<T>` is returned. As it contains only the
/// Id of the widget, it is copyable.
///
/// The widget itself can then be borrowed from the `UI` using
/// [`UI::widget<T: Widget>()`](../ui/struct.UI.html#method.widget),
/// which takes a generic `WidgetHandle` as an argument. So it can
/// deduce and downcast the widget to the actual `T`.
pub struct WidgetHandle<W: Widget> {
    id: Id,
    widget_type: PhantomData<W>
}

impl<W: Widget> Copy for WidgetHandle<W> { }

impl<W: Widget> Clone for WidgetHandle<W> {
    fn clone(&self) -> WidgetHandle<W> {
        WidgetHandle::<W> {
            id: self.id,
            widget_type: PhantomData::<W>
        }
    }
}

impl<W: Widget> WidgetHandle<W> {
    pub(crate) fn new(id: Id) -> Self {
        WidgetHandle::<W> {
            id,
            widget_type: PhantomData::<W>
        }
    }

    pub(crate) fn id(&self) -> Id { self.id }
}

/// Implements [`Widget::stub()`](widget/trait.Widget.html#tymethod.stub)
/// and [`Widget::stub_mut()`](widget/trait.Widget.html#tymethod.stub_mut)
///
/// Assumes that the trait object implementing
/// [`Widget`](widget/trait.Widget.html) has an instance of
/// [`WidgetStub`](widget/struct.WidgetStub.html) in a field `stub`.
#[macro_export]
macro_rules! widget_stub {
    () => {
        fn stub (&self) -> &$crate::widget::WidgetStub {
            &self.stub
        }
        fn stub_mut (&mut self) -> &mut $crate::widget::WidgetStub {
            &mut self.stub
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct DummyWidget {
        stub: WidgetStub
    }

    impl Widget for DummyWidget {
        widget_stub!();
    }

    #[test]
    fn widget_needs_repaint() {
        let mut widget = DummyWidget::default();
        assert!(!widget.needs_repaint());
        widget.ask_for_repaint();
        assert!(widget.needs_repaint());
        assert!(!widget.needs_repaint());
    }

    #[test]
    fn widget_set_focus_repaint() {
        let mut widget = DummyWidget::default();
        assert!(!widget.needs_repaint());
        widget.set_focus(true);
        assert!(widget.needs_repaint());
        widget.set_focus(false);
        assert!(widget.needs_repaint());
    }

    #[test]
    fn widget_set_focus_twice_true() {
        let mut widget = DummyWidget::default();
        assert!(!widget.needs_repaint());
        widget.set_focus(true);
        assert!(widget.needs_repaint());
        widget.set_focus(true);
        assert!(!widget.needs_repaint());
    }

    #[test]
    fn widget_set_focus_twice_false() {
        let mut widget = DummyWidget::default();
        assert!(!widget.needs_repaint());
        widget.set_focus(false);
        assert!(!widget.needs_repaint());
    }

    #[test]
    fn widget_pointer_enter_repaint() {
        let mut widget = DummyWidget::default();
        widget.pointer_enter_wrap();
        assert!(widget.needs_repaint());
    }

    #[test]
    fn widget_pointer_leave_repaint() {
        let mut widget = DummyWidget::default();
        widget.pointer_leave_wrap();
        assert!(widget.needs_repaint());
    }
}
