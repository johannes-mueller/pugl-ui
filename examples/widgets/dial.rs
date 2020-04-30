use std::f64::consts::PI;

use pugl_ui::ui::*;
use pugl_ui::widget::*;
use pugl_sys::*;

pub struct Dial {
    stub: WidgetStub,
    radius: f64,

    value: f64,
    min_value: f64,
    max_value: f64,
    step: f64
}

impl Widget for Dial {
    fn exposed (&self, _exposed: &ExposeArea, cr: &cairo::Context) {

	let pos = self.pos() + Coord { x: self.radius, y: self.radius };

	cr.set_source_rgb(0.7, 0.7, 0.7);
	cr.arc(pos.x, pos.y, self.radius * 0.8, 0.0, 2.*PI);
	cr.fill();

	cr.set_source_rgb(0., 0., 0.);
	cr.set_line_width(self.radius * 0.2);
	cr.arc(pos.x, pos.y, self.radius, 0.0, 2.*PI);
	cr.stroke();

	let angle = 120. + 300. * (self.value-self.min_value)/(self.max_value-self.min_value);
	cr.set_source_rgb(1., 1., 1.);
	cr.set_line_width(self.radius * 0.2);
	cr.arc(pos.x, pos.y, self.radius, (angle-10.0) * PI/180., (angle+10.0) * PI/180.);
	cr.stroke();
    }

    fn event(&mut self, ev: Event) -> Option<Event> {
	match ev.data {
	    EventType::Scroll (sc) => {
		let nv = self.value + sc.dy.signum() * self.step;
		let new_value = match nv {
		    v if v > self.max_value => self.max_value,
		    v if v < self.min_value => self.min_value,
		    _ => nv
		};
		if new_value != self.value {
		    self.ask_for_repaint();
		}
		self.value = new_value;
		event_processed!()
	    }
	    _ => event_not_processed!()
	}.and_then (|p| p.pass_event(ev))
    }

    fn min_size(&self) -> Size {
	Size { w: 2.* self.radius, h: 2.* self.radius }
    }
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
}

impl Dial {
    pub fn set_value(&mut self, v: f64) {
	self.value = v;
	self.ask_for_repaint();
    }

    pub fn value(&self) -> f64 {
	self.value
    }
}

pub struct Factory { min: f64, max: f64, step: f64 }

impl WidgetFactory<Dial> for Factory {
    fn make_widget(&self, stub: WidgetStub) -> Dial {
	Dial {
	    stub, radius: 18.0,
	    value: self.min,
	    min_value: self.min, max_value: self.max,
	    step: self.step
	}
    }
}

pub fn new(min: f64, max: f64, step: f64) -> Factory {
    Factory { min, max, step }
}
