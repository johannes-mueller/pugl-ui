
use pango;

use pugl_ui::ui::*;
use pugl_sys::*;
use pugl_ui::widget::*;


pub struct Button {
    stub: WidgetStub,
    min_size: Size,
    text: String,

    clicked: bool
}

impl Widget for Button {
    fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
        let (r, g, b) = (0.7, 0.7, 0.7);
        let size = self.size();
        let pos = self.pos();

        cr.set_source_rgb (r, g, b);
        cr.rectangle (pos.x, pos.y, size.w, size.h);
        cr.fill ();

        cr.set_source_rgb (0., 0., 0.);

        cr.save();
        cr.translate(pos.x, pos.y);

        let ctx = pangocairo::functions::create_context (&cr).unwrap();
        let lyt = pango::Layout::new (&ctx);

        let font_desc = pango::FontDescription::from_string ("Sans 24px");

        lyt.set_font_description (Some(&font_desc));
        lyt.set_text (&self.text);

        pangocairo::functions::show_layout (cr, &lyt);

        cr.restore();

        if self.has_focus() {
            cr.set_source_rgb (1., 1., 1.);
            cr.rectangle(pos.x, pos.y, size.w, size.h);
            cr.stroke();
        }
    }
    fn event (&mut self, ev: Event) -> Option<Event> {
        match ev.data {
            EventType::MouseMove (_mm) => {
                event_processed!()
            }
            EventType::MouseButtonRelease (_btn) => {
		self.clicked = true;
                event_processed!()
            },
            EventType::KeyRelease (ke) => {
                ke.try_char().and_then(|c| {
                    match c {
                        ' ' => {
                            event_processed!()
                        },
                        _ => event_not_processed!()
                    }
                }).or (event_not_processed!())
            },
            _ => event_not_processed!()
        }.and_then (|es| es.pass_event (ev))
    }
    fn min_size(&self) -> Size { self.min_size }
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }

    fn takes_focus(&self) -> bool { true }
}

impl Button {
    pub fn new (text: &str) -> Box<Button> {
        let sf = cairo::ImageSurface::create (cairo::Format::ARgb32, 8, 8).unwrap();
        let cr = cairo::Context::new (&sf);

        let ctx = pangocairo::functions::create_context (&cr).unwrap();
        let lyt = pango::Layout::new (&ctx);

        let font_desc = pango::FontDescription::from_string ("Sans 24px");

        lyt.set_font_description (Some(&font_desc));
        lyt.set_text (text);

        let (w, h) = lyt.get_pixel_size();
        let min_size: Size = Size { w: w.into(), h: h.into() };

        Box::new(Button {
	    stub: WidgetStub::default(),
	    text: String::from(text),
	    min_size,
	    clicked: false
	})
    }

    pub fn clicked(&mut self) -> bool {
	let clicked = self.clicked;
	self.clicked = false;
	clicked
    }
}
