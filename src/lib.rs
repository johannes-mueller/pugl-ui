//! [`pugl`](https://gitlab.com/lv2/pugl/) is a minimal portable API
//! for embeddable GUIs. This crate aims to provide a stub for
//! GUI-toolkits using `pugl`
//!
//! `pugl-ui` (this crate) features
//! * Widget layouting
//! * Event propagation
//! * Interaction with the windowing system via [`pugl-sys`](https://crates.io/crates/pugl-sys) and
//!   [`pugl`](https://gitlab.com/lv2/pugl/).
//!
//! It does not feature actual widgets, though.
//!
//!
//! # API principles
//!
//! `pugl-ui`'s API differs from classical object oriented approaches
//! of GUI programming. This is due to Rust's safe ownership concepts
//! which disallows shared mutable references to objects.
//!
//! For example if a click on a button was to change the state of
//! something in the app, usually the button would retain a reference
//! or a callback to this "something". When the button is clicked it
//! can use this reference to perform the state change.
//!
//! In Rust that's not possible as the consequence of the button
//! retaining a mutable reference to the state would be that no other
//! reference – not even a readable one – could coexist in the
//! application.
//!
//! ## The players: Widgets, the UI, the Application
//!
//! `pugl-ui` has in principle three players.
//!
//! * **The widgets**: they receive event notifications and can then
//!   change their internal state. Widgets must implement
//!   [`Widget`](widget/trait.Widget.html)
//!
//! * **The UI**: an instance of [`UI`](ui/struct.UI.html)
//!
//!   The UI is the interface between the application, the windowing
//!   system and the widgets.  It receives event notifications from
//!   the windowing system and passes them to the widgets.  Then the
//!   application can borrow references to individual widgets to check
//!   if the application's state needs to be changed.
//!
//! * **The application** holds a reference to the UI and implements
//!   the event loop. There is no trait nor struct for it in
//!   `pugl-ui`.  Typically its a function that initializes the UI and
//!   then has an event loop that asks the `UI` to propagate events
//!   from the windowing system by calling
//!   [`UI::next_event()`](ui/struct.UI.html#method.next_event) and
//!   then checks the widgets if any application state change is
//!   required, for example when a button has been clicked. So it is
//!   the application that holds all the application logic. The
//!   application can also borrow mutable references to widgets, for
//!   example to change their state.
//!
//! ## Widget handles
//!
//! The application does not retain references to the widget. It is
//! the `UI` that has them. The application retains only
//! [`WidgetHandle`](widget/struct.WidgetHandle.html) objects. The
//! `WidgetHandle` object are created by
//! [`UI::new_widget()`](ui/struct.UI.html#method.new_widget) and can
//! later be accessed by
//! [`UI::widget()`](ui/struct.UI.html#method.widget).
//!
//!
//! # Example
//!
//! ```
//! use pugl_sys::*;
//! use pugl_ui::ui::*;
//! use pugl_ui::layout::stacklayout::*;
//! use pugl_ui::widget::*;
//! use pugl_ui::*;
//! use cairo;
//!
//! // A simple root widget, that does only draw a gray background.
//! #[derive(Default)]
//! struct RootWidget {
//!     stub: WidgetStub,
//! }
//!
//! impl Widget for RootWidget {
//!     widget_stub!();
//!     fn exposed (&mut self, _expose: &ExposeArea, cr: &cairo::Context) {
//!         cr.set_source_rgb(0.2, 0.2, 0.2);
//!         let size = self.size();
//!         cr.rectangle(0., 0., size.w, size.h);
//!         cr.fill();
//!     }
//! }
//!
//! const BUTTON_TEXT: &'static str = "Click me";
//!
//! // A simple button that knows when it has been clicked
//! #[derive(Default)]
//! struct Button {
//!     stub: WidgetStub,
//!     clicked: bool,
//! }
//!
//! impl Button {
//!     // by this method the application can check if the button has been clicked
//!     fn has_been_clicked(&mut self) -> bool {
//!         let clicked = self.clicked;
//!         self.clicked = false;
//!         clicked
//!     }
//! }
//!
//! impl Widget for Button {
//!     widget_stub!();
//!
//!     // rendering the button
//!     fn exposed(&mut self, _expose: &ExposeArea, cr: &cairo::Context) {
//!         cr.set_source_rgb(0.7, 0.7, 0.7);
//!         let (x, y, w, h) = self.rect();
//!         cr.rectangle(x, y, w, h);
//!         cr.fill();
//!
//!         cr.set_source_rgb(0., 0., 0.);
//!         cr.move_to(x+w/3., y+2.*h/3.);
//!         cr.select_font_face("Sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
//!         cr.set_font_size(60.0);
//!         cr.show_text(BUTTON_TEXT);
//!         cr.fill();
//!     }
//!
//!     // processing the event
//!     fn event(&mut self, ev: Event) -> Option<Event> {
//!         match ev.data {
//!             EventType::MouseButtonRelease(_) => {
//!                 self.clicked = true;
//!                 event_processed!()
//!             }
//!             _ => event_not_processed!()
//!         }.and_then(|p| p.pass_event(ev))
//!     }
//!
//!     // signaling the minimal size of the button
//!     fn min_size(&self) -> Size {
//!         Size { w: 600., h: 100. }
//!     }
//! }
//!
//! // The application function
//! fn app_execute() {
//!     // Initializing the UI and the interface to the windowing system
//!     let rw = Box::new(RootWidget::default());
//!     let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new(pv, rw));
//!     let ui = view.handle();
//!
//!     // creating the button
//!     let button = ui.new_widget(Box::new(Button::default()));
//!
//!     // widget layouting
//!     ui.pack_to_layout(button, ui.root_layout(), StackDirection::Back);
//!     ui.do_layout();
//!
//!     // showing the window
//!     ui.fit_window_size();
//!     ui.show_window();
//!
//!     // event loop
//!     while !ui.close_request_issued() {
//!         ui.next_event(-1.0);
//!
//!         // minimalist application logic
//!         //
//!         // We borrow the `button` widget from the `ui` and check if it has been clicked.
//!         if ui.widget(button).has_been_clicked() {
//!             println!("Button has been clicked.");
//!         }
//!     }
//! }
//! ```
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
    use crate::layout::stacklayout::*;
    use crate::widget::*;
    use cairo;

    #[derive(Default)]
    struct RootWidget {
        stub: WidgetStub,
        wants_quit: bool,
        focus_next: bool
    }

    impl Widget for RootWidget {
        widget_stub!();
        fn exposed (&mut self, _expose: &ExposeArea, cr: &cairo::Context) {
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
        #[cfg(not(feature = "testing"))]
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

        clicked: bool,
        pointer_entered: bool
    }

    impl Widget for RectWidget {
        widget_stub!();
        fn exposed (&mut self, _expose: &ExposeArea, cr: &cairo::Context) {
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

        fn reminder_handler(&mut self) -> bool {
            self.recently_clicked = false;
            self.ask_for_repaint();
            false
        }

        fn min_size(&self) -> Size { self.min_size }

        fn width_expandable(&self) -> bool {
            self.width_expandable
        }

        fn height_expandable(&self) -> bool {
            self.height_expandable
        }

        fn takes_focus(&self) -> bool { true }

        fn pointer_enter(&mut self) {
            self.pointer_entered = true;
            println!("pointer enter {}", self.name);
        }

        fn pointer_leave(&mut self) {
            self.pointer_entered = false;
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

        #[cfg(feature = "testing")]
        pub fn pointer_in(&self) -> bool {
            self.pointer_entered
        }
    }

    #[cfg(feature = "testing")]
    #[test]
    fn root_widget_quit_event() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.5));

        view.queue_event(Event {
            data: EventType::KeyPress(Key {
                key: KeyVal::Character('q'),
                modifiers: 0,
                code: 0
            }),
            context: EventContext::default()
        });

        let ui = view.handle();
        assert!(!ui.root_widget().wants_quit());
        ui.update(-1.0);
        assert!(ui.root_widget().wants_quit());
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_single_widget_unscaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let expected_size = Size { w: 42., h: 23. };
        {
            let ui = view.handle();
            let widget = ui.new_widget(Box::new(RectWidget {
                min_size: expected_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget).pos(), Coord::default());
            assert_eq!(ui.get_frame().size, expected_size);

        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_single_widget_padding_unscaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 52., h: 33. };
        {
            let ui = view.handle();
            let widget = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(5.).set_spacing(10.);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget).pos(), Coord { x: 5.0, y: 5.0 });
            assert_eq!(ui.get_frame().size, expected_size);
        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_single_widget_scaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 2.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 84., h: 46. };
        {
            let ui = view.handle();
            let widget = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget).pos(), Coord::default());
            assert_eq!(ui.get_frame().size, expected_size);

        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_single_widget_padding_scaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 2.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 104., h: 66. };
        {
            let ui = view.handle();
            let widget = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(5.).set_spacing(10.);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget).pos(), Coord { x: 5.0, y: 5.0 });
            assert_eq!(ui.get_frame().size, expected_size);
        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    #[should_panic(expected = "Root window size zero. Have you forgotten ui::UI::do_layout()?")]
    fn layout_single_widget_no_layout_window_size() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let expected_size = Size { w: 42., h: 23. };
        let ui = view.handle();
        let widget = ui.new_widget(Box::new(RectWidget {
            min_size: expected_size,
            ..Default::default()
        }));
        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
        ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
        ui.fit_window_size();
    }

    #[cfg(feature = "testing")]
    #[test]
    #[should_panic(expected = "Minimal root size zero. Have you forgotten ui::UI::do_layout()?")]
    fn layout_single_widget_no_layout_min_size() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let expected_size = Size { w: 42., h: 23. };
        let ui = view.handle();
        let widget = ui.new_widget(Box::new(RectWidget {
            min_size: expected_size,
            ..Default::default()
        }));
        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
        ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
        ui.fit_window_min_size();
    }

    #[cfg(feature = "testing")]
    #[test]
    #[should_panic(expected = "widget already layouted?")]
    fn layout_same_widget_twice() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let expected_size = Size { w: 42., h: 23. };
        {
            let ui = view.handle();
            let widget = ui.new_widget(Box::new(RectWidget {
                min_size: expected_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
        }
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_two_widgets() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 42., h: 46. };
        {
            let ui = view.handle();
            let widget_1 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            let widget_2 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget_1).pos(), Coord { x: 0.0, y: 23.0 });
            assert_eq!(ui.widget(widget_2).pos(), Coord::default());

            assert_eq!(ui.get_frame().size, expected_size);

        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_two_widgets_spacing() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 42., h: 56. };
        {
            let ui = view.handle();
            let widget_1 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            let widget_2 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(10.);
            ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget_1).pos(), Coord { x: 0.0, y: 33.0 });
            assert_eq!(ui.widget(widget_2).pos(), Coord::default());

            assert_eq!(ui.widget(widget_1).size(), widget_size);
            assert_eq!(ui.widget(widget_2).size(), widget_size);

            assert_eq!(ui.get_frame().size, expected_size);
        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_two_widgets_scaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 2.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 84., h: 92. };
        {
            let ui = view.handle();
            let widget_1 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            let widget_2 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget_1).pos(), Coord { x: 0.0, y: 23.0 });
            assert_eq!(ui.widget(widget_2).pos(), Coord::default());

            assert_eq!(ui.widget(widget_1).size(), widget_size);
            assert_eq!(ui.widget(widget_2).size(), widget_size);

            assert_eq!(ui.get_frame().size, expected_size);

        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_two_widgets_spacing_scaled() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 2.));

        let widget_size = Size { w: 42., h: 23. };
        let expected_size = Size { w: 84., h: 112. };
        {
            let ui = view.handle();
            let widget_1 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            let widget_2 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(10.);
            ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.fit_window_min_size();
            ui.show_window();

            assert_eq!(ui.widget(widget_1).pos(), Coord { x: 0.0, y: 33.0 });
            assert_eq!(ui.widget(widget_2).pos(), Coord::default());

            assert_eq!(ui.get_frame().size, expected_size);

        }
        assert_eq!(view.mock_instance().min_size(), expected_size);
    }

    #[cfg(feature = "testing")]
    #[test]
    fn layout_single_layout_resize() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };
        let (widget_1, widget_2, widget_3, widget_4) = {
            let ui = view.handle();
            let widget_1 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                ..Default::default()
            }));
            let widget_2 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                width_expandable: true,
                ..Default::default()
            }));
            let widget_3 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                height_expandable: true,
                ..Default::default()
            }));
            let widget_4 = ui.new_widget(Box::new(RectWidget {
                min_size: widget_size,
                width_expandable: true,
                height_expandable: true,
                ..Default::default()
            }));
            ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
            ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_3, ui.root_layout(), StackDirection::Front);
            ui.pack_to_layout(widget_4, ui.root_layout(), StackDirection::Front);
            ui.do_layout();
            ui.fit_window_size();
            ui.make_resizable();
            ui.fit_window_min_size();
            ui.show_window();

            (widget_1, widget_2, widget_3, widget_4)
        };

        assert_eq!(view.handle().widget(widget_1).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_2).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_3).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_4).size(), Size { w: 42., h: 23. });

        view.fake_resize(Size { w: 42., h: 102. });

        assert_eq!(view.handle().widget(widget_1).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_2).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_3).size(), Size { w: 42., h: 28. });
        assert_eq!(view.handle().widget(widget_4).size(), Size { w: 42., h: 28. });

        view.fake_resize(Size { w: 42., h: 1. });

        assert_eq!(view.handle().widget(widget_1).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_2).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_3).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_4).size(), Size { w: 42., h: 23. });

        view.fake_resize(Size { w: 50., h: 92. });

        assert_eq!(view.handle().widget(widget_1).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_2).size(), Size { w: 50., h: 23. });
        assert_eq!(view.handle().widget(widget_3).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_4).size(), Size { w: 50., h: 23. });

        view.fake_resize(Size { w: 1., h: 1. });

        assert_eq!(view.handle().widget(widget_1).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_2).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_3).size(), Size { w: 42., h: 23. });
        assert_eq!(view.handle().widget(widget_4).size(), Size { w: 42., h: 23. });

    }

    #[cfg(feature = "testing")]
    #[test]
    fn two_widgets_clicks() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };

        view.queue_event(Event {
            data: EventType::MouseButtonRelease(MouseButton { num: 1, modifiers: 0 }),
            context: EventContext { pos: Coord{ x: 21., y: 11.5 }, ..Default::default() }
        });

        view.queue_event(Event {
            data: EventType::MouseButtonRelease(MouseButton { num: 1, modifiers: 0 }),
            context: EventContext { pos: Coord{ x: 21., y: 28. }, ..Default::default() }
        });

        view.queue_event(Event {
            data: EventType::MouseButtonRelease(MouseButton { num: 1, modifiers: 0 }),
            context: EventContext { pos: Coord{ x: 21., y: 46. }, ..Default::default() }
        });

        let ui = view.handle();
        let widget_1 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        let widget_2 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(10.);
        ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
        ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
        ui.do_layout();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.show_window();

        assert!(!ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());

        ui.update(-1.0);
        assert!(!ui.widget(widget_1).clicked());
        assert!(ui.widget(widget_2).clicked());

        ui.update(-1.0);
        assert!(!ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());

        ui.update(-1.0);
        assert!(ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());
    }

    #[cfg(feature = "testing")]
    #[test]
    fn focus_two_widgets() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_size = Size { w: 42., h: 23. };

        let ui = view.handle();
        let widget_1 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        let widget_2 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
        ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
        ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
        ui.do_layout();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.show_window();

        assert!(!ui.widget(widget_1).has_focus());
        assert!(!ui.widget(widget_2).has_focus());

        ui.focus_next_widget();

        assert!(ui.widget(widget_1).has_focus());
        assert!(!ui.widget(widget_2).has_focus());

        ui.focus_next_widget();

        assert!(!ui.widget(widget_1).has_focus());
        assert!(ui.widget(widget_2).has_focus());

        ui.focus_next_widget();

        assert!(ui.widget(widget_1).has_focus());
        assert!(!ui.widget(widget_2).has_focus());
    }


    #[cfg(feature = "testing")]
    #[test]
    fn test_focus() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        assert!(!view.handle().has_focus());
        view.fake_focus_in();
        assert!(view.handle().has_focus());
        view.fake_focus_out();
        assert!(!view.handle().has_focus());
    }

    #[cfg(feature = "testing")]
    #[test]
    fn two_widgets_focus() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let space_event = Event {
            data: EventType::KeyRelease(Key { key: KeyVal::Character(' '), modifiers: 0, code: 0 }),
            context: EventContext::default()
        };

        view.queue_event(space_event);
        view.queue_event(space_event);
        view.queue_event(space_event);

        let ui = view.handle();
        let widget_1 = ui.new_widget(Box::new(RectWidget::default()));
        let widget_2 = ui.new_widget(Box::new(RectWidget::default()));

        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(10.);

        assert!(!ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());

        ui.update(-1.0);
        assert!(!ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());

        ui.focus_next_widget();
        ui.update(-1.0);
        assert!(ui.widget(widget_1).clicked());
        assert!(!ui.widget(widget_2).clicked());

        ui.focus_next_widget();
        ui.update(-1.0);
        assert!(!ui.widget(widget_1).clicked());
        assert!(ui.widget(widget_2).clicked());
    }

    #[cfg(feature = "testing")]
    #[test]
    fn two_widgets_hover() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let widget_1_hover_event = Event {
            data: EventType::MouseMove(MotionContext::default()),
            context: EventContext { pos: Coord{ x: 21., y: 36. }, ..Default::default() }
        };
        let widget_2_hover_event = Event {
            data: EventType::MouseMove(MotionContext::default()),
            context: EventContext { pos: Coord{ x: 21., y: 11.5 }, ..Default::default() }
        };

        view.queue_event(widget_1_hover_event);
        view.queue_event(widget_2_hover_event);
        view.queue_event(widget_1_hover_event);

        let widget_size = Size { w: 42., h: 23. };

        let ui = view.handle();
        let widget_1 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        let widget_2 = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));
        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
        ui.pack_to_layout(widget_1, ui.root_layout(), StackDirection::Front);
        ui.pack_to_layout(widget_2, ui.root_layout(), StackDirection::Front);
        ui.do_layout();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.show_window();

        assert!(!ui.widget(widget_1).is_hovered());
        assert!(!ui.widget(widget_1).pointer_in());
        assert!(!ui.widget(widget_2).is_hovered());
        assert!(!ui.widget(widget_2).pointer_in());

        ui.update(1.0);

        assert!(ui.widget(widget_1).is_hovered());
        assert!(ui.widget(widget_1).pointer_in());
        assert!(!ui.widget(widget_2).is_hovered());
        assert!(!ui.widget(widget_2).pointer_in());

        ui.update(1.0);

        assert!(!ui.widget(widget_1).is_hovered());
        assert!(!ui.widget(widget_1).pointer_in());
        assert!(ui.widget(widget_2).is_hovered());
        assert!(ui.widget(widget_2).pointer_in());

        ui.update(1.0);

        assert!(ui.widget(widget_1).is_hovered());
        assert!(ui.widget(widget_1).pointer_in());
        assert!(!ui.widget(widget_2).is_hovered());
        assert!(!ui.widget(widget_2).pointer_in());
    }

    #[cfg(feature = "testing")]
    #[test]
    fn hover_pointer_enter_leave_window() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.));

        let enter_event = Event {
            data: EventType::PointerIn,
            context: EventContext { pos: Coord{ x: 21., y: 13. }, ..Default::default() }
        };
        let leave_event = Event {
            data: EventType::PointerOut,
            context: EventContext { pos: Coord{ x: 21., y: 13. }, ..Default::default() }
        };

        view.queue_event(enter_event);
        view.queue_event(leave_event);

        let widget_size = Size { w: 42., h: 23. };

        let ui = view.handle();
        let widget = ui.new_widget(Box::new(RectWidget {
            min_size: widget_size,
            ..Default::default()
        }));

        ui.layouter(ui.root_layout()).set_padding(0.).set_spacing(0.);
        ui.pack_to_layout(widget, ui.root_layout(), StackDirection::Front);
        ui.do_layout();
        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.show_window();

        assert!(!ui.widget(widget).is_hovered());
        assert!(!ui.widget(widget).pointer_in());
        ui.update(-1.0);
        assert!(ui.widget(widget).is_hovered());
        assert!(ui.widget(widget).pointer_in());
        ui.update(-1.0);
        assert!(!ui.widget(widget).is_hovered());
        assert!(!ui.widget(widget).pointer_in());
    }



    #[cfg(all(not(feature = "testing"), test))]
    #[test]
    fn make_window() {
        let rw = Box::new(RootWidget::default());
        let mut view = PuglView::new(std::ptr::null_mut(), |pv| UI::new_scaled(pv, rw, 1.5));
        let ui = view.handle();

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

        ui.layouter(ui.root_layout()).set_padding(30.).set_spacing(20.);
        ui.layouter(blue_red_lt).set_spacing(0.).set_padding(0.);
        ui.layouter(green_yellow_lt).set_spacing(10.0).set_padding(0.);
        ui.layouter(pink_orange_gray_lt).set_padding(0.0).set_spacing(15.0);

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

        ui.fit_window_size();
        ui.fit_window_min_size();
        ui.set_window_title("Test Pugl");
        ui.make_resizable();
        ui.show_window();

        while !(ui.close_request_issued() || ui.root_widget().wants_quit()) {
            ui.next_event(-1.0);

            if ui.widget(red).clicked() {
                println!("Click received red widget");
            }

            if ui.widget(yellow).is_hovered() {
                ui.set_cursor(pugl_sys::Cursor::Hand);
            } else {
                ui.set_cursor(pugl_sys::Cursor::Arrow);
            }

            if ui.root_widget().focus_next() {
                ui.focus_next_widget();
            }
        }
    }
}
