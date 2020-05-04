
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
            cr.set_source_rgb (0.2, 0.2, 0.2);
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

	width_expandable: bool,
	height_expandable: bool,

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

	fn width_expandable(&self) -> bool {
	    self.width_expandable
	}

	fn height_expandable(&self) -> bool {
	    self.height_expandable
	}

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
	width_expandable: bool,
	height_expandable: bool,
        name: &'static str
    }
    impl WidgetFactory<RectWidget> for RectWidgetFactory {
        fn make_widget(&self, stub: WidgetStub) -> RectWidget {
            RectWidget {
                stub: stub,
                color: self.color,
                min_size: self.size,
		width_expandable: self.width_expandable,
		height_expandable: self.height_expandable,
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

        let red = ui.new_widget (RectWidgetFactory {
            color: (1., 0., 0.),
            size: Size { w: 128., h: 64. },
	    width_expandable: false,
	    height_expandable: false,
            name: "ruĝa"
        });

        let blue = ui.new_widget (RectWidgetFactory {
            color: (0., 0., 1.),
            size: Size { w: 128., h: 64. },
	    width_expandable: false,
	    height_expandable: false,
            name: "blua"
        });

	let green = ui.new_widget (RectWidgetFactory {
            color: (0., 1., 0.),
            size: Size { w: 128., h: 64. },
	    width_expandable: false,
	    height_expandable: false,
            name: "verda"
        });

	let yellow = ui.new_widget (RectWidgetFactory {
            color: (1., 1., 0.),
            size: Size { w: 128., h: 64. },
	    width_expandable: true,
	    height_expandable: false,
            name: "flava"
        });

	let pink = ui.new_widget (RectWidgetFactory {
            color: (1., 0., 1.),
            size: Size { w: 128., h: 64. },
	    width_expandable: false,
	    height_expandable: true,
            name: "viola"
        });

	let orange = ui.new_widget (RectWidgetFactory {
            color: (1., 0.5, 0.),
            size: Size { w: 128., h: 64. },
	    width_expandable: true,
	    height_expandable: false,
            name: "oranĝa"
        });

	let light_gray = ui.new_widget (RectWidgetFactory {
            color: (0.8, 0.8, 0.8),
            size: Size { w: 128., h: 64. },
	    width_expandable: true,
	    height_expandable: false,
            name: "hela"
        });

	let mid_gray = ui.new_widget (RectWidgetFactory {
            color: (0.6, 0.6, 0.6),
            size: Size { w: 128., h: 64. },
	    width_expandable: true,
	    height_expandable: true,
            name: "mezhela"
        });

	let dark_gray = ui.new_widget (RectWidgetFactory {
            color: (0.4, 0.4, 0.4),
            size: Size { w: 128., h: 64. },
	    width_expandable: false,
	    height_expandable: false,
            name: "malhela"
        });

	let white = ui.new_widget (RectWidgetFactory {
            color: (1., 1., 1.),
            size: Size { w: 32., h: 32. },
	    width_expandable: false,
	    height_expandable: true,
            name: "b"
        });



	let cyan = ui.new_widget (RectWidgetFactory {
            color: (0., 1., 1.),
            size: Size { w: 512., h: 128. },
	    width_expandable: false,
	    height_expandable: false,
            name: "Eĥoŝanĝo ĉiuĵaŭde"
        });


	let blue_red_lt = ui.new_layouter::<HorizontalLayouter>();
	let green_yellow_lt = ui.new_layouter::<HorizontalLayouter>();
	let pink_orange_gray_lt = ui.new_layouter::<HorizontalLayouter>();
	let gray_lt = ui.new_layouter::<VerticalLayouter>();

	ui.layouter_handle(ui.root_layout()).set_padding(0.).set_spacing(10.);
	ui.layouter_handle(blue_red_lt).set_spacing(0.).set_padding(0.);
	ui.layouter_handle(green_yellow_lt).set_spacing(10.0).set_padding(0.);


        ui.pack_to_layout(cyan, ui.root_layout(), StackDirection::Front);

	ui.pack_to_layout(green_yellow_lt.widget(), ui.root_layout(), StackDirection::Back);
        ui.pack_to_layout(blue_red_lt.widget(), ui.root_layout(), StackDirection::Back);
	ui.pack_to_layout(red, blue_red_lt, StackDirection::Back);
	let sp = ui.new_spacer();
	ui.pack_to_layout(sp, blue_red_lt, StackDirection::Back);
        ui.pack_to_layout(blue, blue_red_lt, StackDirection::Back);

	ui.pack_to_layout(green, green_yellow_lt, StackDirection::Front);
	ui.pack_to_layout(yellow, green_yellow_lt, StackDirection::Front);

	ui.pack_to_layout(pink_orange_gray_lt.widget(), ui.root_layout(), StackDirection::Back);
	ui.pack_to_layout(pink, pink_orange_gray_lt, StackDirection::Back);
	ui.pack_to_layout(orange, pink_orange_gray_lt, StackDirection::Back);
	//let sp = ui.new_spacer();
	//ui.pack_to_layout(sp, pink_orange_gray_lt, StackDirection::Back);
	ui.pack_to_layout(gray_lt.widget(), pink_orange_gray_lt, StackDirection::Back);
	ui.pack_to_layout(white, pink_orange_gray_lt, StackDirection::Back);

	ui.pack_to_layout(mid_gray, gray_lt, StackDirection::Back);
	ui.pack_to_layout(light_gray, gray_lt, StackDirection::Back);
	ui.pack_to_layout(dark_gray, gray_lt, StackDirection::Front);

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
