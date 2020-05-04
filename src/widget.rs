
use downcast_rs::DowncastSync;

use pugl_sys::pugl::*;

pub type Id = usize;

pub trait Widget : DowncastSync {
    fn event (&mut self, ev: Event) -> Option<Event> {
        Some (ev)
    }
    fn exposed (&self, _expose: &ExposeArea, _cr: &cairo::Context) {}

    fn stub (&self) -> &WidgetStub;
    fn stub_mut (&mut self) -> &mut WidgetStub;

    fn size (&self) -> Size {
        let size = self.stub().layout.size;
        size
    }

    fn min_size(&self) -> Size { Default::default() }

    fn sized_width(&self) -> bool {
	self.min_size().w > 0.0
    }

    fn sized_height(&self) -> bool {
	self.min_size().h > 0.0
    }

    fn width_expandable (&self) -> bool { false }
    fn height_expandable (&self) -> bool { false }

    fn set_width (&mut self, width: f64) {
        self.stub_mut().layout.size.w = width;
    }

    fn set_height (&mut self, height: f64) {
        self.stub_mut().layout.size.h = height;
    }

    fn expand_width (&mut self, ammount: f64) {
        self.stub_mut().layout.size.w += ammount;
    }

    fn expand_height (&mut self, ammount: f64) {
        self.stub_mut().layout.size.h += ammount;
    }

    fn set_pos (&mut self, pos: &Coord) {
        self.stub_mut().layout.pos = *pos;
    }

    fn set_size (&mut self, size: &Size) {
        self.stub_mut().layout.size = *size;
    }

    fn set_layout(&mut self, layout: &Layout) {
        self.stub_mut().layout = *layout;
    }

    fn pos (&self) -> Coord {
        let pos = self.stub().layout.pos;
        pos
    }

    fn takes_focus (&self) -> bool {
        false
    }

    fn is_hit_by (&self, pos: Coord) -> bool {
        let layout = self.stub().layout;

        let x1 = layout.pos.x;
        let x2 = x1 + layout.size.w;
        let y1 = layout.pos.y;
        let y2 = y1 + layout.size.h;
        (pos.x > x1 && pos.x < x2) && (pos.y > y1 && pos.y < y2)
    }

    fn set_focus(&mut self, yn: bool) {
        let hf = self.stub().has_focus;
        self.stub_mut().has_focus = yn;
        if hf != yn {
	    self.stub_mut().needs_repaint = true;
        }
    }

    fn pointer_enter(&mut self) {}

    fn pointer_leave(&mut self) {}

    fn needs_repaint(&mut self) -> bool {
	self.stub_mut().needs_repaint()
    }

    fn ask_for_repaint(&mut self)  {
	self.stub_mut().needs_repaint = true;
    }

    fn has_focus (&self) -> bool {
        self.stub().has_focus
    }
}
impl_downcast!(sync Widget);

#[derive(PartialEq)]
pub enum Request2UI {
    Nothing,
    Quit,
    NeedRepaint,
    FocusNextWidget
}

pub trait WidgetFactory<T: Widget> {
    fn make_widget (&self, stub: WidgetStub) -> T;
}

#[derive(Copy, Clone, Default)]
pub struct Layout {
    pub pos: Coord,
    pub size: Size
}

pub struct WidgetStub {
    pub layout: Layout,
    has_focus: bool,
    needs_repaint: bool
}

impl WidgetStub {
    pub fn new() -> WidgetStub {
        WidgetStub {
            has_focus: false,
            layout: Default::default(),
	    needs_repaint: true
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
