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

    fn pack<T: Layouter>(&mut self, widget: Id, mut parent: LayoutWidgetHandle<T>, target: T::Target) {
	let subnode_id = match self.children.iter().position(|ref node| node.id == widget) {
            Some(id) => id,
            None => {
                return;
            }
        };
	let layout_impl_0 = &mut self.layouter.as_deref_mut().expect("::pack(), no layouter found");
	let layout_impl_1 = layout_impl_0.downcast_mut::<T::Implementor>();

	let layout_impl_2 = layout_impl_1.expect("downcast of layouter failed");

	parent.layouter().pack(layout_impl_2, subnode_id, target);
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
}


pub struct UI {
    widgets: Vec<Box<dyn Widget>>,
    root_widget: WidgetNode,
    unlayouted_nodes: HashMap<Id, WidgetNode>,
    root_widget_handle: LayoutWidgetHandle<VerticalLayouter>,
    view: PuglViewFFI,
    focused_widget: Id,
    close_request_issued: bool,
}

impl UI {
    pub fn new<T, F> (factory: F) -> UI
    where T: Widget + 'static,
	  F: WidgetFactory<T> {
        let stub = WidgetStub::new ();
        let root_widget = Box::new(factory.make_widget(stub));
	let root_widget_handle = LayoutWidgetHandle::<VerticalLayouter>::new(0);
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
            close_request_issued: false,
        }
    }

    pub fn new_widget<T, F>(&mut self, factory: F) -> Id
    where T: Widget + 'static,
	  F: WidgetFactory<T> {
        let id = self.widgets.len();

        let stub = WidgetStub::new ();
        self.widgets.push(Box::new(factory.make_widget(stub)));

	self.unlayouted_nodes.insert(id, WidgetNode::new(id));

	id
    }

    pub fn new_layouter<T: Layouter>(&mut self) -> LayoutWidgetHandle<T> {
	let lw = self.new_widget(LayoutWidgetFactory {});
	self.set_layouter::<T>(lw)
    }

    pub fn set_layouter<T: Layouter>(&mut self, id: Id) -> LayoutWidgetHandle<T> {
	self.find_node(id).set_layouter(T::new_implementor());
	LayoutWidgetHandle::<T>::new(id)
    }

    pub fn pack_to_layout<T: Layouter>(&mut self, widget: Id, parent: LayoutWidgetHandle<T>, target: T::Target) {

	let new_node = self.unlayouted_nodes.remove(&widget).expect("widget already layouted?");

        let node = self.find_node(parent.widget());

        node.children.push (new_node);

        node.pack(widget, parent, target);
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

    pub fn root_layout(&self) -> LayoutWidgetHandle<VerticalLayouter> {
	self.root_widget_handle
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
