//#![feature(get_type_id)]

use std::collections::VecDeque;
use std::ptr;

use cairo;

use pugl_sys::pugl::*;

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

type Spacing = f64;

pub enum LayoutDirection {
    Front,
    Back
}

#[derive(Debug)]
pub struct StackLayouter {
    padding: Spacing,
    spacing: Spacing,
    subnodes: VecDeque<Id>
}

impl StackLayouter {
    pub fn new(padding: Spacing, spacing: Spacing) -> StackLayouter {
        StackLayouter {
            padding,
            spacing,
            subnodes: VecDeque::new()
        }
    }
}

impl Default for StackLayouter {
    fn default() -> StackLayouter {
        StackLayouter::new(10.0, 5.0)
    }
}

#[derive(Debug)]
pub enum Layouter {
    None,
    Vertical(StackLayouter),
    Horizontal(StackLayouter),
//    Grid(Spacing, Spacing)
}

pub enum LayoutTarget {
    Vertical(LayoutDirection),
    Horizontal(LayoutDirection)
}

#[derive(Debug)]
struct WidgetNode {
    id: Id,
    layouter: Layouter,
    children: Vec<WidgetNode>
}

impl WidgetNode {
    fn new(id: Id) -> WidgetNode {
        WidgetNode {
            id,
            layouter: Layouter::None,
            children: Vec::new()
        }
    }

    fn new_layouting(id: Id, layouter: Layouter) -> WidgetNode {
        WidgetNode {
            id,
            layouter,
            children: Vec::new()
        }
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

    fn pack(&mut self, widget: Id, target: LayoutTarget) {
        let subnode_id = match self.children.iter().position(|ref node| node.id == widget) {
            Some(id) => id,
            None => {
                return;
            }
        };
        match self.layouter {
            Layouter::Vertical(ref mut sl) => {
                if let LayoutTarget::Vertical(direction) = target {
                    match direction {
                        LayoutDirection::Back  => sl.subnodes.push_back(subnode_id),
                        LayoutDirection::Front => sl.subnodes.push_front(subnode_id)
                    };
                } else {
                    panic!("Received non vertical target for vertical layout")
                };
            },
            Layouter::Horizontal(ref mut sl) => {
                if let LayoutTarget::Horizontal(direction) = target {
                    match direction {
                        LayoutDirection::Back  => sl.subnodes.push_back(subnode_id),
                        LayoutDirection::Front => sl.subnodes.push_front(subnode_id)
                    };
                } else {
                    panic!("Received non horizontal target for horizontal layout")
                };
            },
            Layouter::None => {
                panic!("Packing to none layouter requested!")
            }
        }
    }

    fn apply_sizes (&self, widgets: &mut Vec<Box<dyn Widget>>, orig_pos: Coord) {
        let size_avail = widgets[self.id].size();

        match self.layouter {
            Layouter::Horizontal(ref sl) => {
                let sized_widgets = sl.subnodes.iter().fold (0, | acc, sn | {
                    if widgets[self.children[*sn].id].min_size().w > 0.0 {
                        acc + 1
                    } else {
                        acc
                    }
                });
                let width_avail = size_avail.w - sl.spacing * sized_widgets as f64 - 2.*sl.padding;
                let height_avail = size_avail.h - 2.*sl.padding;
                let (expanders, width_avail) = sl.subnodes.iter().fold((0, width_avail), |(exp, wa), sn| {
                    let wgt = &widgets[self.children[*sn].id];
                    (if wgt.width_expandable() { exp + 1 } else { exp },  wa - wgt.size().w)
                });
                let expand_each = width_avail / expanders as f64;

                let mut pos = orig_pos + Coord { x: sl.padding, y: sl.padding };
                for sn in sl.subnodes.iter() {
                    let wsize = {
                        let widget = &mut widgets[self.children[*sn].id];
                        if widget.width_expandable() {
                            widget.expand_width(expand_each);
                        }
                        if widget.height_expandable() {
                            widget.set_height(height_avail);
                        }
                        widget.set_pos (&pos);
                        widget.size()
                    };
                    self.children[*sn].apply_sizes(widgets, pos);
                    if wsize.w > 0.0 {
                        pos += Coord { x: wsize.w + sl.spacing, y: 0.0 };
                    }
                }
            },
            Layouter::Vertical(ref sl) => {
                let sized_widgets = sl.subnodes.iter().fold (0, | acc, sn | {
                    if widgets[self.children[*sn].id].min_size().h > 0.0 {
                        acc + 1
                    } else {
                        acc
                    }
                });
                let height_avail = size_avail.h - sl.spacing * sized_widgets as f64 - 2.*sl.padding;
                let width_avail = size_avail.w - 2.*sl.padding;
                let (expanders, height_avail) = sl.subnodes.iter().fold((0, height_avail), |(exp, wa), sn| {
                    let wgt = &widgets[self.children[*sn].id];
                    (if wgt.height_expandable() { exp + 1 } else { exp },  wa - wgt.size().h)
                });
                let expand_each = height_avail / expanders as f64;

                let mut pos = Coord { x: sl.padding, y: sl.padding };
                for sn in sl.subnodes.iter() {
                    let wsize = {
                        let widget = &mut widgets[self.children[*sn].id];
                        if widget.height_expandable() {
                            widget.expand_height(expand_each);
                        }
                        if widget.width_expandable() {
                            widget.set_width(width_avail);
                        }
                        widget.set_pos (&pos);
                        widget.size()
                    };
                    self.children[*sn].apply_sizes(widgets, pos);
                    if wsize.h > 0.0 {
                        pos += Coord { x: 0.0, y: wsize.h + sl.spacing };
                    }
                }
            },
            Layouter::None => {}
        }
    }

    fn calc_widget_sizes (&self, widgets: &mut Vec<Box<dyn Widget>>) -> Size {
        if self.children.is_empty() {
            let wgt = &mut widgets[self.id];
            let size = wgt.min_size();
            wgt.set_size(&size);
            return size;
        }

        let mut need = Size::default();

        match self.layouter {
            Layouter::Horizontal(ref sl) => {
                need.w += sl.padding;
                for subnode in sl.subnodes.iter() {

                    let size = self.children[*subnode].calc_widget_sizes(widgets);
                    need.w += size.w;
                    if size.h > need.h {
                        need.h = size.h;
                    }
                    need.w += sl.spacing;
                }
                need.w += sl.padding - sl.spacing;
                need.h += 2.*sl.padding;

                widgets[self.id].set_size(&need);
            },
            Layouter::Vertical(ref sl) => {
                need.h += sl.padding;
                for subnode in sl.subnodes.iter() {

                    let size = self.children[*subnode].calc_widget_sizes(widgets);
                    need.h += size.h;
                    if size.w > need.w {
                        need.w = size.w;
                    }
                    need.h += sl.spacing
                }
                need.w += 2.*sl.padding;
                need.h += sl.padding - sl.spacing;

                widgets[self.id].set_size(&need);
            },
            Layouter::None => {
                panic!("Non layouter called, this shouldn't happen. Or should it?");
            }
        }
        need
    }
}


pub struct UI {
    widgets: Vec<Box<dyn Widget>>,
    root_widget: WidgetNode,
    view: PuglViewFFI,
    focused_widget: Id,
    close_request_issued: bool,
}

impl UI {
    pub fn new<T, F> (factory: F, layouter: Layouter) -> UI
    where T : Widget + 'static, F: WidgetFactory<T> {
        let stub = WidgetStub::new ();
        let root_widget = Box::new(factory.make_widget(stub));
        UI {
            view: ptr::null_mut(),
            root_widget: WidgetNode {
                id: 0,
                layouter,
                children: vec![]
            },
            focused_widget: 0,
            widgets: vec![root_widget],
            close_request_issued: false,
        }
    }

    pub fn new_layouting_widget<T, F>(&mut self, parent: Id, layouter: Layouter, factory: F) -> Id
    where T : Widget + 'static, F: WidgetFactory<T> {
        let id = self.widgets.len();
        if parent >= id {
            panic!("invalid parent");
        }
        let path = VecDeque::new();
        let (path, _) = self.root_widget.search(path, parent);
        {
            let node = self.root_widget.get_node_by_path(path);

            match node.layouter {
                Layouter::None => panic!("Request to add widget to non layouting node"),
                _ => {}
            };

            node.children.push (WidgetNode::new_layouting(id, layouter));
        }
        let stub = WidgetStub::new();
        self.widgets.push(Box::new(factory.make_widget(stub)));

        id
    }

    pub fn new_widget<T, F>(&mut self, parent: Id, factory: F) -> Id
    where T : Widget + 'static, F: WidgetFactory<T> {
        let id = self.widgets.len();
        if parent >= id {
            panic!("invalid parent");
        }
        let path = VecDeque::new();
        let (path, _) = self.root_widget.search(path, parent);

        let node = self.root_widget.get_node_by_path(path);
        node.children.push (WidgetNode::new(id));
        let stub = WidgetStub::new ();

        self.widgets.push(Box::new(factory.make_widget(stub)));

        id
    }

    pub fn pack_to_layout(&mut self, widget: Id, target: LayoutTarget) {
        let path = VecDeque::new();
        let (mut path, _) = self.root_widget.search(path, widget);
        path.pop_back();

        let node = if path.is_empty() {
            &mut self.root_widget
        } else {
            self.root_widget.get_node_by_path(path)
        };
        node.pack(widget, target);
    }

    pub fn do_layout(&mut self) {
        let orig_size = self.widgets[0].size();
        let new_size = {
            let widgets = &mut self.widgets;
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
        let size = self.widgets[0].size();
	if size.h * size.w == 0.0 {
	    panic!("Root window size zero. Have you forgotten ui::UI::do_layout()?");
	}
        self.set_frame(size.w, size.h);
    }

    pub fn fit_window_min_size(&self) {
        let size = self.widgets[0].size();
	if size.h * size.w == 0.0 {
	    panic!("Minimal root size zero. Have you forgotten ui::UI::do_layout()?");
	}
        self.set_min_size(size.w as i32, size.h as i32);
    }

    pub fn close_request_issued(&self) -> bool {
	self.close_request_issued
    }

    pub fn widget<T>(&mut self, id: Id) -> &mut T
    where T: Widget {
	self.widgets[id].downcast_mut::<T>().expect("Widget cast failed!")
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
	println!("Focusing widget {}", fw);
        self.widgets[self.focused_widget].set_focus(false);
        self.focused_widget = fw;
        self.widgets[self.focused_widget].set_focus(true);
    }

    pub fn next_event(&mut self, timeout: f64) {
	for w in self.widgets.iter_mut() {
	    if w.needs_repaint() {
		self.post_redisplay();
		break;
	    }
	}
	self.update(timeout);
    }

    fn pass_exposed(&self, node: &WidgetNode, expose: &ExposeArea, cr: &cairo::Context) {
        self.widgets[node.id].exposed (expose, cr);
        for c in node.children.iter() {
            self.pass_exposed(c, expose, cr);
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
}


pub trait UITrait {
    fn ui(&self) -> &UI;
    fn ui_mut(&mut self) -> &mut UI;

    fn run(&mut self);
}


impl PuglViewTrait for UI {
    fn exposed (&mut self, expose: &ExposeArea, cr: &cairo::Context) {
        self.pass_exposed(&self.root_widget, expose, cr);
    }

    fn event (&mut self, ev: Event) -> Status {
        let ev = match self.widgets[0].event(ev) {
            Some(ev) => ev,
            None => return Status::Success
        };
        let ev = match ev.data {
            EventType::KeyPress (_) |
            EventType::KeyRelease (_) => {
                match self.widgets[self.focused_widget].event(ev) {
                    Some(ev) => ev,
                    None => return Status::Success
                }
            },
            _ => ev
        };

        let mut event_path = self.event_path(&self.root_widget, ev.context.pos, VecDeque::new());
        let mut evop = Some(ev);
        while let Some(id) = event_path.pop_back() {
            evop = match evop {
                Some(ev) => {
                    let ev = self.widgets[id].event(ev);
                    ev
                },
                None => break
            }
        }

	Status::Success
    }

    fn resize (&mut self, size: Size) {
        self.widgets[0].set_size (&size);
        self.do_layout();
    }

    fn close_request (&mut self) {
        self.close_request_issued = true;
    }

    fn set_view (&mut self, v: PuglViewFFI) {
        self.view = v;
    }
    fn view (&self) -> PuglViewFFI {
        self.view
    }
}

impl Drop for UI {
    fn drop(&mut self) {
	eprintln!("Dropping UI");
    }
}
