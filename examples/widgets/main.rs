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

#[derive(Default)]
struct RootWidget {
    stub: WidgetStub,
    wants_quit: bool,
}

impl Widget for RootWidget {
    widget_stub!();

    fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
        cr.set_source_rgb (0., 1., 0.);
        let size = self.size();
        cr.rectangle (0., 0., size.w, size.h);
        cr.fill ();
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
}

fn main() {
    let mut ui = Box::new(UI::new(Box::new(RootWidget::default())));

    let top_layout = ui.new_layouter::<HorizontalLayouter>();
    let dial_layout = ui.new_layouter::<HorizontalLayouter>();

    let dial1 = ui.new_widget(dial::Dial::new(0.0, 1.0, 0.1));
    let dial2 = ui.new_widget(dial::Dial::new(0.0, 1.0, 0.1));
    let dial3 = ui.new_widget(dial::Dial::new(0.0, 1.0, 0.1));

    let reset_button = ui.new_widget(button::Button::new("Reset"));

    println!("starting layouts");

    ui.pack_to_layout(dial1, dial_layout, StackDirection::Front);
    ui.pack_to_layout(dial2, dial_layout, StackDirection::Front);
    ui.pack_to_layout(dial3, dial_layout, StackDirection::Front);

    ui.pack_to_layout(top_layout.widget(), ui.root_layout(), StackDirection::Front);
    ui.pack_to_layout(dial_layout.widget(), ui.root_layout(), StackDirection::Front);

    ui.add_spacer(top_layout, StackDirection::Back);
    ui.pack_to_layout(reset_button, top_layout, StackDirection::Back);
    ui.add_spacer(top_layout, StackDirection::Back);

    ui.do_layout();


    println!("setting up view");

    let view = PuglView::make_view(ui, std::ptr::null_mut());

    let ui = view.handle();

    ui.fit_window_size();
    ui.fit_window_min_size();
    ui.set_window_title("pugl-rs demo");
    ui.show_window();

    println!("starting event looop");
    while !(ui.close_request_issued() || ui.root_widget().wants_quit()) {
	ui.next_event(-1.0);

	if ui.widget(reset_button).clicked() {
	    ui.widget(dial1).set_value(0.0);
	    ui.widget(dial2).set_value(0.0);
	    ui.widget(dial3).set_value(0.0);
	}
    }
}
