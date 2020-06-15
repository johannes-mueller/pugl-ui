
extern crate cairo;
extern crate pango;

extern crate pugl_sys;

#[macro_use]
extern crate downcast_rs;

#[macro_use]
pub mod widget;

#[macro_use]
pub mod ui;
pub mod layout;


#[macro_export]
macro_rules! event_processed { () => (Some($crate::ui::EventState::Processed)) }
#[macro_export]
macro_rules! event_not_processed { () => (Some($crate::ui::EventState::NotProcessed)) }


#[cfg(test)]
mod tests {
    use pugl_sys::*;
    use crate::ui::*;
    use crate::layout::*;
    use crate::widget::*;
    use cairo;

    #[derive(Default)]
    struct RootWidget {
        stub: WidgetStub,
        wants_quit: bool,
        focus_next: bool
    }

    impl Widget
        for RootWidget {
        widget_stub!();
        fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
            cr.set_source_rgb (0.2, 0.2, 0.2);
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

    #[derive(Default)]
    struct RectWidget {
        stub: WidgetStub,
        color: (f64, f64, f64),
        min_size: Size,
        name: &'static str,

        width_expandable: bool,
        height_expandable: bool,

        drag_ongoing: bool,

        recently_clicked: bool,

        clicked: bool
    }

    impl Widget for RectWidget {
        widget_stub!();
        fn exposed (&self, _expose: &ExposeArea, cr: &cairo::Context) {
            let (r, g, b) = self.color;
            let size = self.size();
            let pos = self.pos();

            cr.set_source_rgb (r, g, b);
            cr.rectangle (pos.x, pos.y, size.w, size.h);
            cr.fill ();

            if self.is_hovered() {
                cr.set_source_rgb(0.5, 0., 0.);
            } else {
                cr.set_source_rgb (0., 0., 0.);
            }

            cr.select_font_face ("Hack", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            cr.set_font_size (20.0);

            let extents = cr.text_extents(self.name);

            cr.save();
            cr.translate (pos.x + (size.w-extents.width)/2., pos.y + (size.h+extents.height)/2.);

            cr.show_text (self.name);

            if self.recently_clicked {
                cr.rectangle(0., -extents.height, extents.width, extents.height);
                cr.stroke();
            }

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
                    if self.drag_ongoing {
                        println!("drag to {}:{} {}", ev.context.pos.x, ev.context.pos.y, self.name);
                    }
                    event_processed!()
                }
                EventType::MouseButtonPress(btn) => {
                    if btn.num == 1 {
                        self.drag_ongoing = true;
                    }
                    event_processed!()
                }
                EventType::MouseButtonRelease (btn) => {
                    if btn.num == 1 {
                        if self.drag_ongoing {
                            println!("drag finished {}", self.name);
                        }
                        self.drag_ongoing = false;
                        self.clicked = true;
                        self.recently_clicked = true;
                        self.request_reminder(2.0);
                        self.ask_for_repaint();
                    }

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

        fn reminder_handler(&mut self) {
            self.recently_clicked = false;
            self.ask_for_repaint();
        }

        fn min_size(&self) -> Size { self.min_size }

        fn width_expandable(&self) -> bool {
            self.width_expandable
        }

        fn height_expandable(&self) -> bool {
            self.height_expandable
        }

        fn takes_focus(&self) -> bool { true }

        fn pointer_leave(&mut self) {
            println!("pointer leave {}", self.name);
        }
    }

    impl RectWidget {
        pub fn clicked(&mut self) -> bool {
            if self.clicked {
                println!("RectWidget::clicked() {}", self.name);
            }
            let clicked = self.clicked;
            self.clicked = false;
            clicked
        }
    }


    #[test]
    fn view_tk() {
        let mut ui = Box::new(UI::new(Box::new(RootWidget::default())));

        let red = ui.new_widget (Box::new(RectWidget {
            color: (1., 0., 0.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "ruĝa",
            ..Default::default()
        }));

        let blue = ui.new_widget (Box::new(RectWidget {
            color: (0., 0., 1.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "blua",
            ..Default::default()
        }));

        let green = ui.new_widget (Box::new(RectWidget {
            color: (0., 1., 0.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "verda",
            ..Default::default()
        }));

        let yellow = ui.new_widget (Box::new(RectWidget {
            color: (1., 1., 0.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: true,
            height_expandable: false,
            name: "flava",
            ..Default::default()
        }));

        let pink = ui.new_widget (Box::new(RectWidget {
            color: (1., 0., 1.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: true,
            name: "viola",
            ..Default::default()
        }));

        let orange = ui.new_widget (Box::new(RectWidget {
            color: (1., 0.5, 0.),
            min_size: Size { w: 128., h: 64. },
            width_expandable: true,
            height_expandable: false,
            name: "oranĝa",
            ..Default::default()
        }));

        let light_gray = ui.new_widget (Box::new(RectWidget {
            color: (0.8, 0.8, 0.8),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "hela",
            ..Default::default()
        }));

        let mid_gray = ui.new_widget (Box::new(RectWidget {
            color: (0.6, 0.6, 0.6),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "mezhela",
            ..Default::default()
        }));

        let dark_gray = ui.new_widget (Box::new(RectWidget {
            color: (0.4, 0.4, 0.4),
            min_size: Size { w: 128., h: 64. },
            width_expandable: false,
            height_expandable: false,
            name: "malhela",
            ..Default::default()
        }));

        let white = ui.new_widget (Box::new(RectWidget {
            color: (1., 1., 1.),
            min_size: Size { w: 32., h: 32. },
            width_expandable: true,
            height_expandable: true,
            name: "b",
            ..Default::default()
        }));


        let cyan = ui.new_widget (Box::new(RectWidget {
            color: (0., 1., 1.),
            min_size: Size { w: 512., h: 128. },
            width_expandable: false,
            height_expandable: false,
            name: "Eĥoŝanĝo ĉiuĵaŭde",
            ..Default::default()
        }));


        let blue_red_lt = ui.new_layouter::<HorizontalLayouter>();
        let green_yellow_lt = ui.new_layouter::<HorizontalLayouter>();
        let pink_orange_gray_lt = ui.new_layouter::<HorizontalLayouter>();
        let gray_lt = ui.new_layouter::<VerticalLayouter>();

        ui.layouter_handle(ui.root_layout()).set_padding(30.).set_spacing(20.);
        ui.layouter_handle(blue_red_lt).set_spacing(0.).set_padding(0.);
        ui.layouter_handle(green_yellow_lt).set_spacing(10.0).set_padding(0.);
        ui.layouter_handle(pink_orange_gray_lt).set_padding(0.0).set_spacing(15.0);

        ui.pack_to_layout(cyan, ui.root_layout(), StackDirection::Front);

        ui.pack_to_layout(green_yellow_lt.widget(), ui.root_layout(), StackDirection::Back);
        ui.pack_to_layout(blue_red_lt.widget(), ui.root_layout(), StackDirection::Back);
        ui.pack_to_layout(red, blue_red_lt, StackDirection::Back);
        ui.add_spacer(blue_red_lt, StackDirection::Back);
        ui.pack_to_layout(blue, blue_red_lt, StackDirection::Back);

        ui.pack_to_layout(green, green_yellow_lt, StackDirection::Front);
        ui.pack_to_layout(yellow, green_yellow_lt, StackDirection::Front);

        ui.pack_to_layout(pink_orange_gray_lt.widget(), ui.root_layout(), StackDirection::Back);
        ui.pack_to_layout(gray_lt.widget(), pink_orange_gray_lt, StackDirection::Back);
        ui.pack_to_layout(pink, pink_orange_gray_lt, StackDirection::Back);
        ui.pack_to_layout(orange, pink_orange_gray_lt, StackDirection::Back);
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

        while !(ui.close_request_issued() || ui.root_widget().wants_quit()) {
            ui.next_event(-1.0);

            if ui.widget(red).clicked() {
                println!("Click received rwidget");
            }

            if ui.root_widget().focus_next() {
                ui.focus_next_widget();
            }
        }
    }
}
