
use std::marker::PhantomData;
use downcast_rs::DowncastSync;

use pugl_sys::pugl::*;

/// The unique Id of a widget
pub type Id = usize;

/// The `Widget` trait.
///
/// Widgets need to implement this trait. Most of the methods have
/// default implementations, so that simple widgets can be easily
/// defined. Eeven layouts are internally treated as widgets.
///
/// Data common to all widgets is kept in the struct
/// [WidgetStub](struct.WidgetStub.html) accessible from the widget by
/// the methods `stub()` and `stub_mut()`.
pub trait Widget : DowncastSync {

    /// Called by the UI to pass an event to the widget.
    ///
    /// The widget is supposed to process the Event and return `None`
    /// if the widget has processed the event. If the widget has not
    /// processed the event it shoud return `Some(ev)` so that the
    /// event can be passed to its parent widget.
    ///
    /// There is [EventState](enum.EventState.html) and the macros
    /// [event_processed!()](macro.event_processed.html) and
    /// [event_not_processed!()](macro.event_not_processed.html) to do this.
    ///
    /// The default implementation just passes the event without touching it.
    ///
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
    /// Default implementation does nothing.
    fn reminder_handler(&mut self) { }

    /// Supposed to return a reference to the `WidgetStub` of the widget
    ///
    /// usually implemented by the macro [widget_stub!()](macro.widget_stub.html).
    fn stub (&self) -> &WidgetStub;

    /// Supposed to return a mutable reference to the `WidgetStub` of the widget.
    ///
    /// Usually implemented by the macro [widget_stub!()](macro.widget_stub.html).
    fn stub_mut (&mut self) -> &mut WidgetStub;

    fn ask_for_repaint(&mut self)  {
        self.stub_mut().needs_repaint = true;
    }

    /// The widget can request a reminder after `timeout`
    /// seconds. When the time has passed `reminder_handler() is
    /// called.
    ///
    /// Usually not to be reimplemented.
    fn request_reminder(&mut self, timeout: f64) {
        self.stub_mut().reminder_request = Some(timeout);
    }

    /// Hands the reminder request over to the UI
    ///
    /// Only to be called by the UI as it consumes the reminder request.
    /// Usually not to be reimplemented.
    fn reminder_request(&mut self) -> Option<f64> {
        self.stub_mut().reminder_request.take()
    }

    /// Returns true iff the widget is currently focused.
    ///
    /// Usually not to be reimplemented.
    fn has_focus (&self) -> bool {
        self.stub().has_focus
    }

    /// Returns the size of the widget after layouting.
    ///
    /// Usually not to be reimplemented.
    fn size (&self) -> Size {
        let size = self.stub().layout.size;
        size
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
    fn set_width (&mut self, width: f64) {
        self.stub_mut().layout.size.w = width;
    }

    /// Sets the actual height of the widget to `height`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    fn set_height (&mut self, height: f64) {
        self.stub_mut().layout.size.h = height;
    }

    /// Expands the width of the widget by `ammount`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    fn expand_width (&mut self, ammount: f64) {
        self.stub_mut().layout.size.w += ammount;
    }

    /// Expands the width of the widget by `ammount`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    fn expand_height (&mut self, ammount: f64) {
        self.stub_mut().layout.size.h += ammount;
    }

    /// Sets the position of the widget to `pos`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    fn set_pos (&mut self, pos: &Coord) {
        self.stub_mut().layout.pos = *pos;
    }


    /// Sets the position of the widget to `size`.
    ///
    /// Usually called by the layouter.
    /// Usually not to be reimplemented.
    fn set_size (&mut self, size: &Size) {
        self.stub_mut().layout.size = *size;
    }

    /// Returns the [Layout](struct.Layout.html) (drawing rect) of the widget.
    ///
    /// Usually not to be reimplemented.
    fn layout(&self) -> Layout {
        self.stub().layout
    }

    /// sets the Layout of the widget to `layout`
    ///
    /// Probably not needed as only used once in UI
    fn set_layout(&mut self, layout: &Layout) {
        self.stub_mut().layout = *layout;
    }

    /// Returns the positon (upper left corner of the widget)
    ///
    /// Usually not to be reimplemented.
    fn pos (&self) -> Coord {
        let pos = self.stub().layout.pos;
        pos
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
    fn is_hovered(&self) -> bool {
        self.stub().hovered
    }

    /// Returns true iff the widget's Layout is containing `pos`.
    ///
    /// Usually not to be reimplemented.
    fn is_hit_by (&self, pos: Coord) -> bool {
        let layout = self.stub().layout;

        let x1 = layout.pos.x;
        let x2 = x1 + layout.size.w;
        let y1 = layout.pos.y;
        let y2 = y1 + layout.size.h;
        (pos.x > x1 && pos.x < x2) && (pos.y > y1 && pos.y < y2)
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
#[derive(Copy, Clone, Default)]
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
/// Contains the Id of the widget and a `PhantomData` of its actual type.
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
    /// Creates a new widget handle for the widget with the Id `id`.
    ///
    /// Called by UI.
    pub(crate) fn new(id: Id) -> Self {
        WidgetHandle::<W> {
            id: id,
            widget_type: PhantomData::<W>
        }
    }

    pub fn id(&self) -> Id { self.id }
}

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
