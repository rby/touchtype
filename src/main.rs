use gtk::prelude::*;
use rand::thread_rng;
use relm4::drawing::DrawHandler;
use relm4::{gtk::Inhibit, prelude::*};
use session::Practice;
use std::path::Path;
use std::time::Instant;

mod msg;
mod session;
mod stats;
use crate::msg::Msg;
use crate::stats::Stats;

#[relm4::component]
impl SimpleComponent for Stats {
    type Init = Stats;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Label {
            #[watch]
            set_label: &format!("{}/s", model.avg_key_s())
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = init;
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::KeyPressed(_, _, _, _) => self.add(msg),
        }
    }
}

#[derive(Debug)]
struct UpdateDrawingMsg;

struct KeyboardState {
    practice: session::Practice,
    handler: DrawHandler,
}

#[relm4::component]
impl SimpleComponent for KeyboardState {
    type Init = session::Practice;
    type Input = Msg;
    type Output = ();

    view! {
            gtk::Box {
                #[local_ref]
                area -> gtk::DrawingArea {
                    set_vexpand: true,
                    set_hexpand: true,
                    inline_css: "border: 2px solid blue",

                },
            },
    }

    fn init(
        practice: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        println!("Received an init");
        let mut handler = DrawHandler::new();
        let cx = handler.get_context();
        let _ = cx.show_text("some text");
        cx.set_source_rgb(0.0, 1.0, 0.0);
        cx.move_to(0.0, 0.0);
        cx.line_to(350.0, 350.0);
        cx.stroke().expect("should draw something");

        let model = KeyboardState { practice, handler };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        println!("Received an update");
        let cx = self.handler.get_context();
        cx.select_font_face(
            "Arial Black",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Bold,
        );
        cx.set_source_rgb(0.0, 0.0, 0.0);
        cx.set_font_size(12.0);
        cx.move_to(10.0, 20.0);

        cx.show_text("some text").expect("awful error");
        cx.fill().expect("should fill something");
    }
}

struct App {
    stats: Controller<Stats>,
    keyboard_state: Controller<KeyboardState>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = Practice;
    type Input = Msg;
    type Output = Msg;

    view! {
        gtk::Window {
            set_title: Some("Type Touching"),
            set_default_size: (800, 640),
            add_controller = gtk::EventControllerKey {
                connect_key_pressed[sender] => move |_, keyval, keycode, state| {
                    let now = Instant::now();
                    sender.input(Msg::KeyPressed(keyval, keycode, state, now));
                    Inhibit(false)
                }
            },
            gtk::Box {
              set_orientation: gtk::Orientation::Vertical,
              set_spacing: 10,
              #[local_ref]
              my_stats -> gtk::Label {set_opacity: 0.7},
              #[local_ref]
              my_ks -> gtk::Box {}
            },
        },

    }

    fn init(
        practice: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats = Stats::builder().launch(Stats::new()).detach();
        let keyboard_state = KeyboardState::builder().launch(practice).detach();
        let model = App {
            stats,
            keyboard_state,
        };
        let my_stats = model.stats.widget();
        let my_ks = model.keyboard_state.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::KeyPressed(_, _, _, _) => {
                self.stats.emit(msg.clone());
                self.keyboard_state.emit(msg);
            }
        }
    }
}

// TODO should be a result later
fn main() {
    let app = RelmApp::new("TouchTyping Master");
    let mut rng = thread_rng();
    let practice =
        session::Practice::generate(&mut rng, 50, Path::new("./data/t8.shakespeare.freq"))
            .expect("should load correctly");
    println!("next practice is : {practice:?}");
    app.run::<App>(practice);
}
