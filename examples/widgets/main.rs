extern crate cairo;
extern crate pango;

#[macro_use]
extern crate pugl_ui;

mod button;
mod dial;

use pugl_ui::widget::*;
use pugl_ui::layout::*;
use pugl_ui::ui::*;

use pugl_sys::*;

struct RootWidget {
    stub: WidgetStub,
    wants_quit: bool,
    focus_next: bool
}

impl Widget for RootWidget {
    fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
        cr.set_source_rgb (0., 1., 0.);
        let size = self.size();
        cr.rectangle (0., 0., size.w, size.h);
        cr.fill ();
    }
    fn min_size(&self) -> Size { Size { w: 0., h: 0. } }
    fn stub (&self) -> &WidgetStub {
        &self.stub
    }
    fn stub_mut (&mut self) -> &mut WidgetStub {
        &mut self.stub
    }
    fn event(&mut self, ev: Event) -> Option<Event> {
        ev.try_keypress()
            .and_then(|kp| kp.try_char())
            .and_then(|c| {
                match c {
                    'q' => {
                        self.wants_quit = true;
                        event_processed!()
                    },
                    '\t' => {
                        self.focus_next = true;
                        event_processed!()
                    },
                    _ => event_not_processed!()
                }
            })
            .or(event_not_processed!()).and_then (|p| p.pass_event (ev))
    }
}

impl RootWidget {
    pub fn wants_quit(&self) -> bool {
	self.wants_quit
    }
    pub fn focus_next(&mut self) -> bool {
	let f = self.focus_next;
	self.focus_next = false;
	f
    }
}

struct RootWidgetFactory {}
impl WidgetFactory<RootWidget> for RootWidgetFactory {
    fn make_widget(&self, stub: WidgetStub) -> RootWidget {
        RootWidget {
            stub,
	    wants_quit: false,
	    focus_next: false
        }
    }
}

fn main() {
    let mut ui = Box::new(UI::new( RootWidgetFactory {}));

    let dial_layout = ui.new_layouter(HorizontalLayouter {});

    let dial1 = ui.new_widget(dial::new(0.0, 1.0, 0.1));
    let dial2 = ui.new_widget(dial::new(0.0, 1.0, 0.1));
    let dial3 = ui.new_widget(dial::new(0.0, 1.0, 0.1));

    let reset_button = ui.new_widget(button::new("Reset"));

    println!("starting layouts");

    ui.pack_to_layout(dial1, dial_layout, StackDirection::Front);
    ui.pack_to_layout(dial2, dial_layout, StackDirection::Front);
    ui.pack_to_layout(dial3, dial_layout, StackDirection::Front);

    ui.pack_to_layout(dial_layout.widget(), ui.root_layout(), StackDirection::Front);
    ui.pack_to_layout(reset_button, ui.root_layout(), StackDirection::Front);

    ui.do_layout();


    println!("setting up view");

    let view = PuglView::make_view(ui, std::ptr::null_mut());

    let ui = view.handle();

    ui.fit_window_size();
    ui.fit_window_min_size();
    ui.set_window_title("pugl-rs demo");
    ui.show_window();

    println!("starting event looop");
    while !(ui.close_request_issued() || ui.widget::<RootWidget>(0).wants_quit()) {
	ui.next_event(-1.0);

	if ui.widget::<button::Button>(reset_button).clicked() {
	    ui.widget::<dial::Dial>(dial1).set_value(0.0);
	    ui.widget::<dial::Dial>(dial2).set_value(0.0);
	    ui.widget::<dial::Dial>(dial3).set_value(0.0);
	}
    }
}
