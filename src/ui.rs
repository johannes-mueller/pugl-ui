//#![feature(get_type_id)]

use std::collections::{VecDeque,HashMap};
use std::ptr;

use cairo;

use pugl_sys::pugl::*;

use crate::layout::*;
use crate::widget::*;

pub enum EventState {
    Processed,
    NotProcessed
}

impl EventState {
    pub fn pass_event (&self, ev: Event) -> Option<Event> {
        match self {
            EventState::Processed => None,
            EventState::NotProcessed => Some(ev)
        }
    }
}

//#[derive(Debug)]
pub struct WidgetNode {
    pub(crate) id: Id,
    layouter: Option<Box<dyn LayouterImpl>>,
    pub(crate) children: Vec<WidgetNode>
}

impl WidgetNode {
    fn new(id: Id) -> WidgetNode {
        WidgetNode {
            id,
            layouter: None,
            children: Vec::new()
        }
    }

    pub fn set_layouter(&mut self, layouter: Box<dyn LayouterImpl>) {
        self.layouter = Some(layouter);
    }

    fn search(&self, path: VecDeque<Id>, id: Id) -> (VecDeque<Id>, bool) {
        if self.id == id {
            return (path, true);
        }
        let mut path = path;
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

    fn get_node_by_path(&mut self, mut path: VecDeque<Id>) -> &mut WidgetNode {
        let id = path.pop_front();
        match id {
            None => self,
            Some(id) => self.children[id].get_node_by_path(path)
        }
    }

    fn layouter_impl<L: Layouter>(&mut self) -> &mut L::Implementor {
        self.layouter
            .as_deref_mut().expect("::pack(), no layouter found")
            .downcast_mut::<L::Implementor>().expect("downcast of layouter failed")
    }

    fn pack<L: Layouter, W: Widget>(&mut self, widget: Id, mut parent: LayoutWidgetHandle<L, W>, target: L::Target) {
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
            layouter.apply_sizes(widgets, &self.children, orig_pos, size_avail);
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
            .calc_widget_sizes(widgets, &self.children);

        widgets[self.id].set_size(&size);

        size
    }

    fn detect_expandables(&self, widgets: &mut Vec<Box<dyn Widget>>) -> (bool, bool) {
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


pub struct UI<RW: Widget + 'static> {
    widgets: Vec<Box<dyn Widget>>,
    root_widget: WidgetNode,
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
    pub fn new(root_widget: Box<RW>) -> UI<RW> {
        let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter, RW>::new(WidgetHandle::<RW>::new(0));
        UI {
            view: ptr::null_mut(),
            root_widget: WidgetNode {
                id: 0,
                layouter: Some(VerticalLayouter::new_implementor()),
                children: vec![]
            },
            unlayouted_nodes: HashMap::new(),
            root_widget_handle,
            focused_widget: 0,
            widgets: vec![root_widget],
            drag_ongoing: false,
            have_focus: false,
            widget_under_pointer: 0,
            close_request_issued: false,

            scale_factor: 1.0
        }
    }

    pub fn new_scaled(root_widget: Box<RW>, scale_factor: f64) -> UI<RW> {
        let mut ui = UI::new(root_widget);
        ui.scale_factor = scale_factor;
        ui
    }

    pub fn new_widget<W: Widget>(&mut self, mut widget: Box<W>) -> WidgetHandle<W> {
        let id = self.widgets.len();
        self.widgets.push(widget);
        self.unlayouted_nodes.insert(id, WidgetNode::new(id));

        WidgetHandle::<W>::new(id)
    }

    pub fn new_layouter<L>(&mut self) -> LayoutWidgetHandle<L, LayoutWidget>
    where L: Layouter {
        let lw = self.new_widget(Box::new(LayoutWidget::default()));
        self.set_layouter::<L>(lw)
    }

    pub fn set_layouter<L>(&mut self, lw: WidgetHandle<LayoutWidget>) -> LayoutWidgetHandle<L, LayoutWidget>
    where L: Layouter {
        self.find_node(lw.id()).set_layouter(L::new_implementor());
        LayoutWidgetHandle::<L, LayoutWidget>::new(lw)
    }

    pub fn add_spacer<L>(&mut self, parent: LayoutWidgetHandle<L, LayoutWidget>, target: L::Target)
    where L: Layouter {
        let sp = self.new_widget(Box::new(Spacer::default()));
        self.pack_to_layout(sp, parent, target);
    }

    pub fn layouter_handle<L, W>(&mut self, layouter: LayoutWidgetHandle<L, W>) -> &mut L::Implementor
    where L: Layouter, W: Widget {
        self.find_node(layouter.widget().id()).layouter_impl::<L>()
    }

    pub fn pack_to_layout<L, W, PW>(&mut self, widget: WidgetHandle<W>, parent: LayoutWidgetHandle<L, PW>, target: L::Target)
    where L: Layouter,
          W: Widget,
          PW: Widget {

        let id = widget.id();
        if let Some(sp) = self.widgets[id].downcast_mut::<Spacer>() {
            sp.set_expandable(L::expandable());
        }

        let new_node = self.unlayouted_nodes.remove(&id).expect("widget already layouted?");
        let node = self.find_node(parent.widget().id());

        node.children.push (new_node);
        node.pack(id, parent, target);
    }

    pub fn do_layout(&mut self) {
        if self.unlayouted_nodes.len() > 0 {
            eprintln!("WARNING: Rendering layout with {} unlayouted widgets!", self.unlayouted_nodes.len());
        }
        let orig_size = self.widgets[0].size();
        let new_size = {
            let widgets = &mut self.widgets;
            self.root_widget.detect_expandables(widgets);
            self.root_widget.calc_widget_sizes(widgets);
            let size = widgets[0].size();
            let new_size = if (orig_size.w > size.w) || (orig_size.h > size.h) {
                orig_size
            } else {
                size
            };
            widgets[0].set_size (&new_size);
            self.root_widget.apply_sizes(widgets, Default::default());
            new_size
        };
        self.widgets[0].set_layout(&Layout { pos: Default::default(), size: new_size });
    }

    pub fn fit_window_size(&self) {
        let size = self.widgets[0].size().scale(self.scale_factor);
        if size.h * size.w == 0.0 {
            panic!("Root window size zero. Have you forgotten ui::UI::do_layout()?");
        }
        self.set_frame(size.w, size.h);
    }

    pub fn fit_window_min_size(&self) {
        let size = self.widgets[0].size().scale(self.scale_factor);
        if size.h * size.w == 0.0 {
            panic!("Minimal root size zero. Have you forgotten ui::UI::do_layout()?");
        }
        self.set_min_size(size.w as i32, size.h as i32);
    }

    pub fn close_request_issued(&self) -> bool {
        self.close_request_issued
    }

    pub fn root_layout(&self) -> LayoutWidgetHandle<VerticalLayouter, RW> {
        self.root_widget_handle
    }

    pub fn root_widget(&mut self) -> &mut RW {
        self.widgets[0].downcast_mut::<RW>().expect("Root Widget cast failed")
    }

    pub fn widget<W: Widget>(&mut self, widget: WidgetHandle<W>) -> &mut W {
        self.widgets[widget.id()].downcast_mut::<W>().expect("Widget cast failed!")
    }

    pub fn focus_next_widget (&mut self) {
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

    pub fn has_focus(&self) -> bool {
        self.have_focus
    }

    pub fn next_event(&mut self, timeout: f64) {
        for (id, w) in self.widgets.iter_mut().enumerate() {
            if w.needs_repaint() {
                let pos = w.pos().scale(self.scale_factor);
                let size = w.size().scale(self.scale_factor);
                self.post_redisplay_rect(pos, size);
                break;
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

    fn event_path(&self, widget: &WidgetNode, pos: Coord, mut path: VecDeque<Id>) -> VecDeque<Id> {
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
                let (path, _) = self.root_widget.search(path, id);
                self.root_widget.get_node_by_path(path)
            }
        }
    }
}



impl<RW: Widget> PuglViewTrait for UI<RW> {
    fn exposed (&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        let mut expose_queue: Vec<Id> = Vec::with_capacity(self.widgets.len());
        cr.scale(self.scale_factor, self.scale_factor);
        self.make_expose_queue(&self.root_widget, expose, &mut expose_queue);
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

        let mut event_path = self.event_path(&self.root_widget, ev.pos(), VecDeque::new());
        let mut evop = Some(ev);

        if let Some(id) = event_path.back() {
            if self.widget_under_pointer != *id {
                self.widgets[self.widget_under_pointer].pointer_leave_wrap();
                self.widgets[*id].pointer_enter_wrap();
                self.widget_under_pointer = *id;
            }
        }

        while let Some(id) = event_path.pop_back() {
            evop = match evop {
                Some(ev) => {
                    let ev = self.widgets[id].event(ev);
                    if let Some(timeout) = self.widgets[id].reminder_request() {
                        self.start_timer(id, timeout);
                    }
                    ev
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
        self.widgets[0].set_size (&size.scale(1./self.scale_factor));
        self.do_layout();
    }

    fn close_request (&mut self) {
        self.close_request_issued = true;
    }

    fn timer_event(&mut self, id: usize) -> Status {
        self.widgets[id].reminder_handler();
        Status::Success
    }

    fn set_view (&mut self, v: PuglViewFFI) {
        self.view = v;
    }
    fn view (&self) -> PuglViewFFI {
        self.view
    }
}
