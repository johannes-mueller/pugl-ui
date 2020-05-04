
use std::collections::VecDeque;

use downcast_rs::DowncastSync;

use pugl_sys::{Coord, Size};
use crate::ui;
use crate::widget::*;


pub trait LayouterImpl: DowncastSync {
    fn apply_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>,
		   orig_pos: Coord, available_size: Size);
    fn calc_widget_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>) -> Size;
}
impl_downcast!(sync LayouterImpl);

pub trait Layouter : Default {
    type Target;
    type Implementor: LayouterImpl;
    fn new_implementor() -> Box<dyn LayouterImpl>;
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: Id, target: Self::Target);
    fn expandable() -> (bool, bool);
}

type Spacing = f64;

pub enum StackDirection {
    Front,
    Back
}

struct StackLayoutData {
    padding: Spacing,
    spacing: Spacing,
    subnodes: VecDeque<Id>,
}

impl Default for StackLayoutData {
    fn default() -> StackLayoutData {
	StackLayoutData {
	    padding: 0.0,
	    spacing: 5.0,
	    subnodes: VecDeque::new(),
	}
    }
}

impl StackLayoutData {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) {
	match target {
            StackDirection::Back  => self.subnodes.push_back(subnode_id),
            StackDirection::Front => self.subnodes.push_front(subnode_id)
        };
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct HorizontalLayouter;

pub struct HorizontalLayouterImpl {
    d: StackLayoutData
}

impl HorizontalLayouterImpl {
    pub fn set_spacing(&mut self, s: Spacing) -> &mut HorizontalLayouterImpl {
	self.d.spacing = s;
	self
    }
    pub fn set_padding(&mut self, s: Spacing) -> &mut HorizontalLayouterImpl {
	self.d.padding = s;
	self
    }
}

impl Default for HorizontalLayouterImpl {
    fn default() -> HorizontalLayouterImpl {
	HorizontalLayouterImpl {
	    d: StackLayoutData::default()
	}
    }
}

impl LayouterImpl for HorizontalLayouterImpl {
    fn apply_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>,
		   orig_pos: Coord, size_avail: Size) {
	let sized_widgets = self.d.subnodes.iter().fold (0, | acc, sn | {
            if widgets[children[*sn].id].min_size().w > 0.0 {
                acc + 1
            } else {
                acc
            }
        });
        let width_avail = size_avail.w - self.d.spacing * (sized_widgets - 1) as f64  - 2.*self.d.padding;
        let height_avail = size_avail.h - 2.*self.d.padding;
        let (expanders, width_avail) = self.d.subnodes.iter().fold((0, width_avail), |(exp, wa), sn| {
            let wgt = &widgets[children[*sn].id];
            (if wgt.width_expandable() { exp + 1 } else { exp },  wa - wgt.size().w)
        });
        let expand_each = width_avail / expanders as f64;

        let mut pos = orig_pos + Coord { x: self.d.padding, y: self.d.padding };
        for sn in self.d.subnodes.iter() {
            let wsize = {
                let widget = &mut widgets[children[*sn].id];
                if widget.width_expandable() {
                    widget.expand_width(expand_each);
                }
                if widget.height_expandable() {
                    widget.set_height(height_avail);
                }
                widget.set_pos (&pos);
                widget.size()
            };
            children[*sn].apply_sizes(widgets, pos);
            if wsize.w > 0.0 {
                pos += Coord { x: wsize.w + self.d.spacing, y: 0.0 };
            }
        }
    }

    fn calc_widget_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>) -> Size {
	let mut need = Size::default();
        need.w += self.d.padding;
        for subnode in self.d.subnodes.iter() {

            let size = children[*subnode].calc_widget_sizes(widgets);
            need.w += size.w;
            if size.h > need.h {
                need.h = size.h;
            }
            need.w += self.d.spacing;
        }
        need.w += self.d.padding - self.d.spacing;
        need.h += 2.*self.d.padding;

	need
    }
}

impl HorizontalLayouterImpl {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) { self.d.pack(subnode_id, target) }
}

impl Layouter for HorizontalLayouter {
    type Target = StackDirection;
    type Implementor = HorizontalLayouterImpl;

    fn new_implementor() -> Box<dyn LayouterImpl> {
	Box::new(HorizontalLayouterImpl::default())
    }
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: Id, target: Self::Target) {
	layout_impl.pack(subnode_id, target);
    }
    fn expandable() -> (bool, bool) {
	(true, false)
    }
}


#[derive(Clone, Copy, Default, Debug)]
pub struct VerticalLayouter;

pub struct VerticalLayouterImpl {
    d: StackLayoutData
}

impl VerticalLayouterImpl {
    pub fn set_spacing(&mut self, s: Spacing) -> &mut VerticalLayouterImpl {
	self.d.spacing = s;
	self
    }
    pub fn set_padding(&mut self, s: Spacing) -> &mut VerticalLayouterImpl {
	self.d.padding = s;
	self
    }
}

impl Default for VerticalLayouterImpl {
    fn default() -> VerticalLayouterImpl {
	VerticalLayouterImpl {
	    d: StackLayoutData::default()
	}
    }
}

impl LayouterImpl for VerticalLayouterImpl {
    fn apply_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>,
		   orig_pos: Coord, size_avail: Size) {
        let sized_widgets = self.d.subnodes.iter().fold (0, | acc, sn | {
            if widgets[children[*sn].id].min_size().h > 0.0 {
                acc + 1
            } else {
                acc
            }
        });
        let height_avail = size_avail.h - self.d.spacing * (sized_widgets - 1) as f64 - 2.*self.d.padding;
        let width_avail = size_avail.w - 2.*self.d.padding;
        let (expanders, height_avail) = self.d.subnodes.iter().fold((0, height_avail), |(exp, wa), sn| {
            let wgt = &widgets[children[*sn].id];
            (if wgt.height_expandable() { exp + 1 } else { exp },  wa - wgt.size().h)
        });
        let expand_each = height_avail / expanders as f64;

        let mut pos = orig_pos + Coord { x: self.d.padding, y: self.d.padding };
        for sn in self.d.subnodes.iter() {
            let wsize = {
                let widget = &mut widgets[children[*sn].id];
                if widget.height_expandable() {
                    widget.expand_height(expand_each);
                }
                if widget.width_expandable() {
                    widget.set_width(width_avail);
                }
                widget.set_pos (&pos);
                widget.size()
            };
            children[*sn].apply_sizes(widgets, pos);
            if wsize.h > 0.0 {
                pos += Coord { x: 0.0, y: wsize.h + self.d.spacing };
            }
        }

    }

    fn calc_widget_sizes(&self, widgets: &mut Vec<Box<dyn Widget>>, children: &Vec<ui::WidgetNode>) -> Size {
	let mut need = Size::default();
        need.h += self.d.padding;
        for subnode in self.d.subnodes.iter() {

            let size = children[*subnode].calc_widget_sizes(widgets);
            need.h += size.h;
            if size.w > need.w {
                need.w = size.w;
            }
            need.h += self.d.spacing
        }
        need.w += 2.*self.d.padding;
        need.h += self.d.padding - self.d.spacing;

	need
    }
}

impl VerticalLayouterImpl {
    fn pack(&mut self, subnode_id: Id, target: StackDirection) { self.d.pack(subnode_id, target) }
}

impl Layouter for VerticalLayouter {
    type Target = StackDirection;
    type Implementor = VerticalLayouterImpl;

    fn new_implementor() -> Box<dyn LayouterImpl> {
	Box::new(VerticalLayouterImpl::default())
    }
    fn pack(&mut self, layout_impl: &mut Self::Implementor, subnode_id: Id, target: Self::Target) {
	layout_impl.pack(subnode_id, target);
    }
    fn expandable() -> (bool, bool) {
	(false, true)
    }
}






pub struct LayoutWidget {
    stub: WidgetStub,
    width_expandable: bool,
    height_expandable: bool
}

impl LayoutWidget {
    pub(crate) fn set_expandable(&mut self, we: bool, he: bool) {
	self.width_expandable = we;
	self.height_expandable = he;
    }
}

impl Widget for LayoutWidget {
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }

    fn width_expandable(&self) -> bool { self.width_expandable }
    fn height_expandable(&self) -> bool { self.height_expandable }
}

pub struct LayoutWidgetFactory {}
impl WidgetFactory<LayoutWidget> for LayoutWidgetFactory {
    fn make_widget(&self, stub: WidgetStub) -> LayoutWidget {
        LayoutWidget {
	    stub,
	    width_expandable: false,
	    height_expandable: false
	}
    }
}

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

pub struct SpacerFactory {}
impl WidgetFactory<Spacer> for SpacerFactory {
    fn make_widget(&self, stub: WidgetStub) -> Spacer {
        Spacer {
	    stub,
	    width_expandable: false,
	    height_expandable: false
	}
    }
}


#[derive(Clone, Copy)]
pub struct LayoutWidgetHandle<T: Layouter> {
    id: Id,
    layouter: T
}

impl<T: Layouter> LayoutWidgetHandle<T> {
    pub fn new(id: Id) -> LayoutWidgetHandle<T> {
	LayoutWidgetHandle { id, layouter: T::default() }
    }
    pub fn widget(&self) -> Id {
	self.id
    }
    pub fn layouter(&mut self) -> &mut T {
	&mut self.layouter
    }
    pub fn expandable() -> (bool, bool) {
	T::expandable()
    }
}
