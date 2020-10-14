//! The `UI` struct and widget management facilities
//!
//! # Principles
//!
//! Widgets are kept in a `Vec<dyn Box<Widget>`. To address an
//! individual widget unambiguously the widgets [`ID`](type.Id.html)
//! is used which directly corresponds to the index of the widget in
//! the `Vec`.
//!
//! Moreover widgets are kept in a hierarchical tree. So each widget,
//! except for the root widget with the `ID` 0 has exactly one parent
//! widget. As of now a widget's geometry is a subset of the parent's
//! geometry. This limitation implicates that for widgets that need to
//! overlay other widgets, like drop down widgets, a new mechanism
//! needs to be implemented, like floating widgets.
//!
//! The widget hierachy tree is used to perform two things.
//!
//! * Perform the widget layouting. Each widget is responsible to
//!   layout its children.
//!
//! * Event propagation. The `UI` finds the widget that recieves the
//!   event, if the widget does not process the event, the event is
//!   propagated to its parent.
//!
use std::collections::{VecDeque,HashMap};

use pugl_sys::*;

use crate::layout::*;
use crate::layout::layoutwidget::*;
use crate::layout::stacklayout::*;
use crate::widget::*;

/// Used to indicate if an event has been processed
pub enum EventState {
    Processed,
    NotProcessed
}

impl EventState {
    /// Returns `Some(ev)` if the `EventState` is `NotProcessed`, otherwise `None`
    pub fn pass_event (&self, ev: Event) -> Option<Event> {
        match self {
            EventState::Processed => None,
            EventState::NotProcessed => Some(ev)
        }
    }
}

/// A node in the widget tree (internal use only)
///
/// See ['layout'](../layout/index.html) for principles about widget layouting.
pub struct WidgetNode {
    pub(crate) id: Id,
    pub(crate) layouter: Option<Box<dyn LayouterImpl>>,
    pub(crate) children: Vec<WidgetNode>
}

impl WidgetNode {
    pub(crate) fn new_leaf(id: Id) -> WidgetNode {
        WidgetNode {
            id,
            layouter: None,
            children: Vec::new()
        }
    }

    pub(crate) fn new_node<L: Layouter>(id: Id) -> WidgetNode {
        WidgetNode {
            id,
            layouter: Some(L::new_implementor()),
            children: Vec::new()
        }
    }

    pub(crate) fn root<L: Layouter>() -> WidgetNode {
        WidgetNode {
            id: 0,
            layouter: Some(L::new_implementor()),
            children: Vec::new()
        }
    }

    /// Recursively completes the path to widget `id``
    ///
    /// The path is the way from `UI::root_widget` following by index
    /// of `WidgetNode::children` Followed by `get_node_by_path()` it is
    /// used to find the `WidgetNode` instance containing a certain
    /// widget `Id`.
    fn search(&self, mut path: VecDeque<usize>, id: Id) -> (VecDeque<usize>, bool) {
        if self.id == id {
            return (path, true);
        }
        for (i, c) in self.children.iter().enumerate() {
            path.push_back(i);
            let (p, found) = c.search(path, id);
            if found {
                return (p, true);
            }
            path = p;
            path.pop_back();
        }
        (path, false)
    }

    /// Recursively follows the path to the end finally returning the
    /// final node. Takes a path set up by `search()`.
    fn get_node_by_path(&mut self, mut path: VecDeque<usize>) -> &mut WidgetNode {
        let index = path.pop_front();
        match index {
            None => self,
            Some(i) => self.children[i].get_node_by_path(path)
        }
    }

    pub(crate) fn layouter_impl<L: Layouter>(&mut self) -> &mut L::Implementor {
        self.layouter
            .as_deref_mut().expect("::pack(), no layouter found")
            .downcast_mut::<L::Implementor>().expect("downcast of layouter failed")
    }

    pub(crate) fn pack<L: Layouter, W: Widget>(&mut self, widget: Id, mut parent: LayoutWidgetHandle<L, W>, target: L::Target) {
        let subnode_id = match self.children.iter().position(|ref node| node.id == widget) {
            Some(id) => id,
            None => {
                return;
            }
        };

        parent.layouter().pack(self.layouter_impl::<L>(), subnode_id, target);
    }

    pub(crate) fn apply_sizes (&self, widgets: &mut Vec<Box<dyn Widget>>, orig_pos: Coord) {
        let size_avail = widgets[self.id].size();

        if let Some(layouter) = &self.layouter {
            layouter.apply_layouts(widgets, &self.children, orig_pos, size_avail);
        }
    }

    pub(crate) fn calc_widget_sizes (&self, widgets: &mut Vec<Box<dyn Widget>>) -> Size {
        if self.children.is_empty() {
            let wgt = &mut widgets[self.id];
            let size = wgt.min_size();
            wgt.set_size(&size);

            return size;
        }

        let size = self.layouter
            .as_ref()
            .expect("::calc_widget_sizes() no layouter found")
            .calc_size(widgets, &self.children);

        widgets[self.id].set_size(&size);

        size
    }

    pub(crate) fn detect_expandables(&self, widgets: &mut Vec<Box<dyn Widget>>) -> (bool, bool) {
        if self.children.is_empty() {
            let wgt = &widgets[self.id];
            return (wgt.width_expandable(), wgt.height_expandable())
        }

        let mut width_exp = false;
        let mut height_exp = false;

        for c in self.children.iter() {
            let (we, he) = c.detect_expandables(widgets);
            width_exp = we || width_exp;
            height_exp = he || height_exp;
        }

        if self.id != 0 {
            let lw = &mut widgets[self.id].downcast_mut::<LayoutWidget>().expect("Downcast to LayoutWidget failed.");
            lw.set_expandable(width_exp, height_exp);
        }
        (width_exp, height_exp)
    }
}

/// The central interface between application, widgets and the windowing system
///
/// The `UI` has the following responsibilities.
///
/// * retain references to the widgets
///
/// * receive events from the windowing system and pass them to the relevant widgets
///
/// * lend widgets to the application, so the application can check
///   the state of the widgets.
///
/// * supervise widget layouting
///
/// # Event propagation
///
/// The `UI` receives events from the windowing system. Depending on
/// the context (mouse cursor position, kind of event, focused widget,
/// ...) the UI passes the event to the widget that should receive the
/// event. If that widget does not processes the event, the event can
/// then be passed to another widget, usually the parent of the
/// widget. This is done by the following rules.
///
/// ## Keyboard events
///
/// Keyboard events are first passed to the root widget, i.e. the
/// widget that has been passed to the constructor of the `UI`.
///
/// If the root widget does not process the event, the event is passed
/// to the focused widget. There are the methods
/// [`focus_widget()`](#method.focus_widget) and
/// [`focus_next_widget()`](#method.focus_next_widget) to set the
/// focus to a specific widget.
///
///
/// ## Mouse events
///
/// A mouse events goes to the widget that is under the mouse
/// pointer. Widgets are kept in a tree of
/// [`WidgetNode`](struct.WidgetNode.html)s. If the widget under the
/// pointer does not process the event, it is passed to its parent.
///
/// ## Exeption: mouse dragging
///
/// When a mouse dragging is ongoing, the widget in which the mouse
/// dragging started, receives, mouse events and key events first,
/// until the dragging stops.
///
pub struct UI<RW: Widget + 'static> {
    widgets: Vec<Box<dyn Widget>>,
    root_widget_node: WidgetNode,
    unlayouted_nodes: HashMap<Id, WidgetNode>,
    root_widget_handle: LayoutWidgetHandle<VerticalLayouter, RW>,
    view: PuglViewFFI,
    focused_widget: Id,
    widget_under_pointer: Id,
    drag_ongoing: bool,
    have_focus: bool,
    close_request_issued: bool,

    scale_factor: f64
}

impl<RW: Widget + 'static> UI<RW> {
    /// Creates a new `UI` instance from a `PuglViewFFI` and a heap allocated root widget
    ///
    /// The UI instance needs a `PuglViewFFI` instance from the
    /// [`pugl-sys`](https://docs.rs/pugl-sys) crate as interface to
    /// the windowing system.
    pub fn new(view: PuglViewFFI, root_widget: Box<RW>) -> UI<RW> {
        UI {
            view,
            root_widget_node: WidgetNode::root::<VerticalLayouter>(),
            unlayouted_nodes: HashMap::new(),
            root_widget_handle: LayoutWidgetHandle::<VerticalLayouter, RW>::new(WidgetHandle::new(0)),
            focused_widget: 0,
            widgets: vec![root_widget],
            drag_ongoing: false,
            have_focus: false,
            widget_under_pointer: 0,
            close_request_issued: false,

            scale_factor: 1.0
        }
    }

    /// Creates a new `UI` which is scaled by the `scale_factor`
    ///
    /// Widgets don't know about the scale factor. They can do their
    /// drawing and event processing as if the `scale_factor` was
    /// `1.0`. The `UI` everything including the `cairo::Context`
    /// transparently.
    pub fn new_scaled(view: PuglViewFFI, root_widget: Box<RW>, scale_factor: f64) -> UI<RW> {
        let mut ui = UI::new(view, root_widget);
        ui.scale_factor = scale_factor;
        ui
    }

    fn push_widget<W: Widget>(&mut self, widget: Box<W>) -> Id {
        let id = self.widgets.len();
        self.widgets.push(widget);
        id
    }

    /// Registers a new widget in the `UI`.
    ///
    /// The instance of the widget must be passed heap allocated in a `Box`.
    /// Returns a `WidgetHandle` to the widget.
    pub fn new_widget<W: Widget>(&mut self, widget: Box<W>) -> WidgetHandle<W> {
        let id = self.push_widget(widget);
        self.unlayouted_nodes.insert(id, WidgetNode::new_leaf(id));

        WidgetHandle::<W>::new(id)
    }

    /// Creates a new `LayoutingWidget` for a `Layouter` of type `L` and registers it to the UI/
    ///
    /// Returns a `LayoutWidgetHandle to the `Layouter` object.
    pub fn new_layouter<L>(&mut self) -> LayoutWidgetHandle<L, LayoutWidget>
    where L: Layouter {
        let id = self.push_widget(Box::new(LayoutWidget::default()));
        self.unlayouted_nodes.insert(id, WidgetNode::new_node::<L>(id));
        LayoutWidgetHandle::<L, LayoutWidget>::new(WidgetHandle::new(id))
    }

    /// Adds a spacing widget to a layouter.
    ///
    /// This is a convenience function
    pub fn add_spacer<L>(&mut self, parent: LayoutWidgetHandle<L, LayoutWidget>, target: L::Target)
    where L: Layouter {
        let sp = self.new_widget(Box::new(Spacer::new(L::expandable())));
        self.pack_to_layout(sp, parent, target);
    }

    /// Adds the `widget` to a `layout` according to the layout
    /// `target`. The `target` is specific to the actual `Layouter` type `L`
    pub fn pack_to_layout<L, W, PW>(&mut self, widget: WidgetHandle<W>, parent: LayoutWidgetHandle<L, PW>, target: L::Target)
    where L: Layouter,
          W: Widget,
          PW: Widget {

        let id = widget.id();

        let new_node = self.unlayouted_nodes.remove(&id).expect("widget already layouted?");
        let node = self.find_node(parent.widget().id());

        node.children.push(new_node);
        node.pack(id, parent, target);
    }

    /// Performs the layouting of the widgets.
    ///
    /// This must be done before the view is realized (or window is
    /// shown). All registered widgets should have been packed to a
    /// layout before.
    pub fn do_layout(&mut self) {
        if !self.unlayouted_nodes.is_empty() {
            eprintln!("WARNING: Rendering layout with {} unlayouted widgets!", self.unlayouted_nodes.len());
        }
        let orig_size = self.widgets[0].size();
        let new_size = {
            let widgets = &mut self.widgets;
            self.root_widget_node.detect_expandables(widgets);
            self.root_widget_node.calc_widget_sizes(widgets);
            let size = widgets[0].size();
            let new_size = if (orig_size.w > size.w) || (orig_size.h > size.h) {
                orig_size
            } else {
                size
            };
            widgets[0].set_size(&new_size);
            self.root_widget_node.apply_sizes(widgets, Default::default());
            new_size
        };
        self.widgets[0].set_layout(&Layout { pos: Default::default(), size: new_size });
    }

    /// Sets the default window size, so that the widget layout fits into it.
    pub fn fit_window_size(&self) {
        let size = self.widgets[0].size().scale(self.scale_factor);
        if size.h * size.w == 0.0 {
            panic!("Root window size zero. Have you forgotten ui::UI::do_layout()?");
        }
        self.set_default_size(size.w as i32, size.h as i32);
    }

    /// Sets the minimal window size, so that the widget layout fits into it.
    pub fn fit_window_min_size(&self) {
        let size = self.widgets[0].size().scale(self.scale_factor);
        if size.h * size.w == 0.0 {
            panic!("Minimal root size zero. Have you forgotten ui::UI::do_layout()?");
        }
        self.set_min_size(size.w as i32, size.h as i32);
    }

    /// Returns `true` iff a the window has been requested to close by the windowing system
    ///
    /// The application should check for this at every cycle of the
    /// event loop and terminate the event loop if `true` is returned.
    pub fn close_request_issued(&self) -> bool {
        self.close_request_issued
    }

    /// Returns a mutable reference to the `Layouter` of the passed `LayoutWidgetHandle`.
    ///
    /// This can be used to borrow a handle to the layouter in order
    /// to change layouting parameters.
    pub fn layouter<L, W>(&mut self, layouter: LayoutWidgetHandle<L, W>) -> &mut L::Implementor
    where L: Layouter, W: Widget {
        self.find_node(layouter.widget().id()).layouter_impl::<L>()
    }

    /// Returns a mutable reference to the `Layouter` of root Layouter.
    ///
    /// This can be used to borrow a handle to the layouter in order
    /// to change layouting parameters.
    pub fn root_layout(&self) -> LayoutWidgetHandle<VerticalLayouter, RW> {
        self.root_widget_handle
    }

    /// Returns a mutable reference to the root widget.
    pub fn root_widget(&mut self) -> &mut RW {
        self.widgets[0].downcast_mut::<RW>().expect("Root Widget cast failed")
    }

    /// Returns a mutable reference to the specified by `widget`.
    ///
    /// It returns a reference to the actual widget instance, so type specific
    /// methods of the widget can be used.
    pub fn widget<W: Widget>(&mut self, widget: WidgetHandle<W>) -> &mut W {
        self.widgets[widget.id()].downcast_mut::<W>().expect("Widget cast failed!")
    }

    /// Performs a step in the cycle of the widget focus.
    ///
    /// Can be called when the root widget received a TAB key press event.
    pub fn focus_next_widget(&mut self) {
        let mut fw = self.focused_widget;
        loop {
            fw += 1;
            if fw == self.widgets.len() {
                fw = 0;
            }
            if self.widgets[fw].takes_focus() || (fw == self.focused_widget) {
                break;
            }
        }

        self.widgets[self.focused_widget].set_focus(false);
        self.focused_widget = fw;
        self.widgets[self.focused_widget].set_focus(true);
    }

    /// Focuses the widget specified by `widget`
    ///
    pub fn focus_widget<W: Widget>(&mut self, widget: WidgetHandle<W>) {
        let id = widget.id();
        if self.widgets[id].takes_focus() {
            self.widgets[self.focused_widget].set_focus(false);
            self.focused_widget = id;
            self.widgets[id].set_focus(true);
        }
    }

    /// Returns `true` iff the window has the focus.
    pub fn has_focus(&self) -> bool {
        self.have_focus
    }

    /// Initiates the next cycle of the event loop
    ///
    /// The application should call it at the beginning of the event loop.
    ///
    /// From `pugl` documentation:
    /// If `timeout` is zero, then this function will not block. Plugins
    /// should always use a timeout of zero to avoid blocking the
    /// host.
    ///
    /// If a positive `timeout` is given, then events will be processed
    /// for that amount of time, starting from when this function was
    /// called.
    ///
    /// If a `negative` timeout is given, this function will block
    /// indefinitely until an event occurs.
    ///
    /// For continuously animating programs, a timeout that is a
    /// reasonable fraction of the ideal frame period should be used,
    /// to minimize input latency by ensuring that as many input
    /// events are consumed as possible before drawing.
    pub fn next_event(&mut self, timeout: f64) {
        for id in 0..self.widgets.len() {
            let w = &mut self.widgets[id]; if w.needs_repaint() {
                let pos = w.pos().scale(self.scale_factor);
                let size = w.size().scale(self.scale_factor);
                self.post_redisplay_rect(pos, size);
            }
            let w = &mut self.widgets[id];
            if let Some(timeout) = w.reminder_request() {
                self.start_timer(id, timeout);
            }
        }
        self.update(timeout);
    }

    fn make_expose_queue(&self, node: &WidgetNode, area: &ExposeArea, expose_queue: &mut Vec<Id>) {
        let pos = area.pos.scale(1./self.scale_factor);
        let size = area.size.scale(1./self.scale_factor);
        if !self.widgets[node.id].intersects_with(pos, size) {
            return;
        }
        expose_queue.push(node.id);
        for c in node.children.iter() {
            self.make_expose_queue(c, area, expose_queue);
        }
    }

    fn event_path(&self, widget: &WidgetNode, pos: Coord, mut path: VecDeque<usize>) -> VecDeque<usize> {
        path.push_back(widget.id);
        for c in widget.children.iter() {
            if self.widgets[c.id].is_hit_by(pos) {
                return self.event_path(c, pos, path);
            }
        }
        path
    }

    fn find_node(&mut self, id: Id) -> &mut WidgetNode {
        match self.unlayouted_nodes.get_mut(&id) {
            Some(l) => l,
            None => {
                let path = VecDeque::new();
                let (path, _) = self.root_widget_node.search(path, id);
                self.root_widget_node.get_node_by_path(path)
            }
        }
    }
}



impl<RW: Widget> PuglViewTrait for UI<RW> {
    fn exposed (&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        let mut expose_queue: Vec<Id> = Vec::with_capacity(self.widgets.len());
        cr.scale(self.scale_factor, self.scale_factor);
        self.make_expose_queue(&self.root_widget_node, expose, &mut expose_queue);
        for wid in expose_queue {
            self.widgets[wid].exposed(expose, cr);
        }
    }

    fn event (&mut self, ev: Event) -> Status {
        let ev = ev.scale_pos(1./self.scale_factor);
        let ev = match self.widgets[0].event(ev) {
            Some(ev) => ev,
            None => return Status::Success
        };
        let ev = match ev.data {
            EventType::KeyPress (_) |
            EventType::KeyRelease (_) => {
                if self.drag_ongoing {
                    self.widgets[self.widget_under_pointer].event(ev);
                    return Status::Success
                }
                match self.widgets[self.focused_widget].event(ev) {
                    Some(ev) => ev,
                    None => return Status::Success
                }
            }
            EventType::MouseButtonPress(btn) => {
                if btn.num == 1 {
                    self.drag_ongoing = true;
                }
                ev
            }
            EventType::MouseButtonRelease(btn) => {
                if btn.num == 1 && self.drag_ongoing {
                    self.drag_ongoing = false;
                    let wgt = &mut self.widgets[self.widget_under_pointer];
                    let pev = wgt.event(ev);
                    if !wgt.is_hit_by(ev.pos()) {
                        wgt.pointer_leave_wrap();
                    }
                    match pev {
                        Some(ev) => ev,
                        None => return Status::Success
                    }
                } else {
                    ev
                }
            }
            _ => {
                if self.drag_ongoing {
                    self.widgets[self.widget_under_pointer].event(ev);
                    return Status::Success;
                }
                ev
            }
        };

        let mut event_path = self.event_path(&self.root_widget_node, ev.pos(), VecDeque::new());
        let mut evop = Some(ev);

        if let Some(id) = event_path.back() {
            if self.widget_under_pointer != *id {
                self.widgets[self.widget_under_pointer].pointer_leave_wrap();
                self.widgets[*id].pointer_enter_wrap();
                self.widget_under_pointer = *id;
            }
            if ev.data == EventType::PointerIn {
                self.widgets[*id].pointer_enter_wrap();
                self.widget_under_pointer = *id;
            }
            if ev.data == EventType::PointerOut {
                self.widgets[self.widget_under_pointer].pointer_leave_wrap();
            }
        }

        while let Some(id) = event_path.pop_back() {
            evop = match evop {
                Some(ev) => {
                    self.widgets[id].event(ev)
                },
                None => break
            }
        }

        Status::Success
    }

    fn focus_in(&mut self) -> Status {
        self.have_focus = true;
        self.widgets[self.focused_widget].set_focus(true);
        Status::Success
    }

    fn focus_out(&mut self) -> Status {
        self.have_focus = false;
        self.widgets[self.focused_widget].set_focus(false);
        Status::Success
    }

    fn resize (&mut self, size: Size) {
        self.widgets[0].set_size(&size.scale(1./self.scale_factor));
        self.do_layout();
    }

    fn close_request (&mut self) {
        self.close_request_issued = true;
    }

    fn timer_event(&mut self, id: usize) -> Status {
        if !self.widgets[id].reminder_handler() {
            self.stop_timer(id);
        }
        Status::Success
    }

    fn view (&self) -> PuglViewFFI {
        self.view
    }
}
