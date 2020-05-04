
extern crate cairo;
extern crate pango;

extern crate pugl_sys;

#[macro_use]
extern crate downcast_rs;

pub mod widget;

#[macro_use]
pub mod ui;
pub mod layout;


#[macro_export]
macro_rules! event_processed { () => (Some(EventState::Processed)) }
#[macro_export]
macro_rules! event_not_processed { () => (Some(EventState::NotProcessed)) }


#[cfg(test)]
mod tests {
    use pugl_sys::*;
    use crate::ui::*;
    use crate::layout::*;
    use crate::widget::*;
    use cairo;

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

    struct RectWidget {
        stub: WidgetStub,
        color: (f64, f64, f64),
        min_size: Size,
        name: &'static str,

	clicked: bool
    }

    impl Widget for RectWidget {
        fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
            let (r, g, b) = self.color;
            let size = self.size();
            let pos = self.pos();

            cr.set_source_rgb (r, g, b);
            cr.rectangle (pos.x, pos.y, size.w, size.h);
            cr.fill ();

            cr.set_source_rgb (0., 0., 0.);

            cr.select_font_face ("Hack", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            cr.set_font_size (20.0);

	    let extents = cr.text_extents(self.name);

            cr.save();
            cr.translate (pos.x + (size.w-extents.width)/2., pos.y + (size.h+extents.height)/2.);

            cr.show_text (self.name);

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
                    println!("mouse move {} {}", ev.context.pos.x, ev.context.pos.y);
                    event_processed!()
                }
                EventType::MouseButtonRelease (btn) => {
                    println!("Button number {}", btn.num);
                    self.clicked = true;
                    event_processed!()
                },
                EventType::KeyRelease (ke) => {
                    println!("Recieved a key release: {}", self.name);
                    ke.try_char().and_then(|c| {
                        match c {
                            ' ' => {
                                self.clicked = true;
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


    struct RectWidgetFactory {
        color: (f64, f64, f64),
        size: Size,
        name: &'static str
    }
    impl WidgetFactory<RectWidget> for RectWidgetFactory {
        fn make_widget(&self, stub: WidgetStub) -> RectWidget {
            RectWidget {
                stub: stub,
                color: self.color,
                min_size: self.size,
                name: self.name,
		clicked: false
            }
        }
    }

    impl RectWidget {
	pub fn clicked(&mut self) -> bool {
	    println!("RectWidget::clicked() {:?}", self.clicked);
	    let clicked = self.clicked;
	    self.clicked = false;
	    clicked
	}
    }


    #[test]
    fn view_tk() {
        let mut ui = Box::new(UI::new(RootWidgetFactory {}));

	let layouter = ui.new_layouter::<HorizontalLayouter>();

        let rw2 = ui.new_widget (RectWidgetFactory {
            color: (1., 0., 0.),
            size: Size { w: 128., h: 64. },
            name: "red"
        });

        let rw4 = ui.new_widget (RectWidgetFactory {
            color: (0., 1., 1.),
            size: Size { w: 512., h: 129. },
            name: "Eĥoŝanĝo ĉiuĵaŭde"
        });


        let sp1 = ui.new_widget (SpacerFactory {});

        ui.pack_to_layout(sp1, layouter, StackDirection::Back);

        ui.pack_to_layout(layouter.widget(), ui.root_layout(), StackDirection::Back);
        ui.pack_to_layout(rw2, layouter, StackDirection::Back);
        ui.pack_to_layout(rw4, ui.root_layout(), StackDirection::Front);
        ui.do_layout();

        let view = PuglView::make_view(ui, std::ptr::null_mut());
	let ui = view.handle();

        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.set_window_title("Test Pugl");
        ui.show_window();

	while !(ui.close_request_issued() || ui.widget::<RootWidget>(0).wants_quit()) {
	    ui.next_event(-1.0);

	    if ui.widget::<RectWidget>(rw2).clicked() {
		println!("Click received rwidget");
	    }

	    if ui.widget::<RootWidget>(0).focus_next() {
		ui.focus_next_widget();
	    }
	}
    }
}
