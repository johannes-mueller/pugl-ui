use std::f64::consts::PI;

use pugl_ui::ui::*;
use pugl_ui::widget::*;
use pugl_sys::*;

#[derive(Default)]
pub struct Dial {
    stub: WidgetStub,
    radius: f64,

    value: f64,
    min_value: f64,
    max_value: f64,
    step: f64,

    value_indicator_active: bool
}

impl Dial {
    pub fn new(min_value: f64, max_value: f64, step: f64) -> Box<Dial> {
	Box::new(Dial { min_value, max_value, step, radius: 18.0, ..Default::default() })
    }
    pub fn set_value(&mut self, v: f64) {
	self.value = v;
	self.ask_for_repaint();
    }

    pub fn value(&self) -> f64 {
	self.value
    }
}

impl Widget for Dial {
    fn exposed (&self, _exposed: &ExposeArea, cr: &cairo::Context) {

	let pos = self.pos() + Coord { x: self.radius, y: self.radius };

	cr.save();
	cr.translate(pos.x + self.radius, pos.y + self.radius);

	cr.set_source_rgb(0.7, 0.7, 0.7);
	cr.arc(0., 0., self.radius * 0.8, 0.0, 2.*PI);
	cr.fill();

	cr.set_source_rgb(0., 0., 0.);
	cr.set_line_width(self.radius * 0.2);
	cr.arc(0., 0., self.radius, 0.0, 2.*PI);
	cr.stroke();

	let angle = 120. + 300. * (self.value-self.min_value)/(self.max_value-self.min_value);
	cr.set_source_rgb(1., 1., 1.);
	cr.set_line_width(self.radius * 0.2);
	cr.arc(0., 0., self.radius, (angle-10.0) * PI/180., (angle+10.0) * PI/180.);
	cr.stroke();

	if self.value_indicator_active {
	    let ctx = pangocairo::functions::create_context(&cr).expect("cration of pango context failed");
	    let lyt = pango::Layout::new(&ctx);
	    let font_desc = pango::FontDescription::from_string("Sans 12px");

	    lyt.set_font_description(Some(&font_desc));
	    lyt.set_text(&format!("{:.1}dB", self.value));

	    let (ent, _) = lyt.get_extents();
	    let (w, h) = ((ent.width/pango::SCALE) as f64, (ent.height/pango::SCALE) as f64);
	    let bl = (lyt.get_baseline()/pango::SCALE) as f64;

	    cr.translate(-w/2., -2.5*self.radius + h);
	    cr.set_source_rgb(0., 0., 0.);
	    cr.rectangle(0., 0., w, h+(bl/2.));
	    cr.fill();
	    cr.set_source_rgb(1., 1., 1.);
	    pangocairo::functions::show_layout(cr, &lyt);
	}
	cr.restore();
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

    fn pointer_enter(&mut self) {
	self.value_indicator_active = true;
	self.ask_for_repaint();
    }

    fn pointer_leave(&mut self) {
	self.value_indicator_active = false;
	self.ask_for_repaint();
    }

    fn min_size(&self) -> Size {
	Size { w: 4. * self.radius, h: 4. * self.radius }
    }
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
}
