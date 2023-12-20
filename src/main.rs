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

const UNIT: f64 = 30.0;
const QWERTY: &[(&str, f64)] = &[
    ("`", 1.0),
    ("1", 1.0),
    ("2", 1.0),
    ("3", 1.0),
    ("4", 1.0),
    ("5", 1.0),
    ("6", 1.0),
    ("7", 1.0),
    ("8", 1.0),
    ("9", 1.0),
    ("0", 1.0),
    ("-", 1.0),
    ("=", 1.0),
    ("del", 1.5), // line 1
    ("tab", 1.5),
    ("Q", 1.0),
    ("W", 1.0),
    ("E", 1.0),
    ("R", 1.0),
    ("T", 1.0),
    ("Y", 1.0),
    ("U", 1.0),
    ("I", 1.0),
    ("O", 1.0),
    ("P", 1.0),
    ("[", 1.0),
    ("]", 1.0),
    ("\\", 1.0), // line 2
    ("caps", 2.0),
    ("A", 1.0),
    ("S", 1.0),
    ("D", 1.0),
    ("F", 1.0),
    ("G", 1.0),
    ("H", 1.0),
    ("J", 1.0),
    ("K", 1.0),
    ("L", 1.0),
    (";", 1.0),
    ("'", 1.0),
    ("enter", 2.0), // line 3
    ("shift", 3.0),
    ("Z", 1.0),
    ("X", 1.0),
    ("C", 1.0),
    ("V", 1.0),
    ("B", 1.0),
    ("N", 1.0),
    ("M", 1.0),
    (",", 1.0),
    (".", 1.0),
    ("/", 1.0),
    ("shift", 3.0), // line 4
    ("space", 10.0),
];
const LAYOUT: &[usize] = &[14, 14, 13, 12, 1];
const HSTART: f64 = 100.0;
const VSTART: f64 = 100.0;

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
    handler: DrawHandler,
}

#[relm4::component]
impl SimpleComponent for KeyboardState {
    type Init = ();
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
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        println!("Received an init");
        let handler = DrawHandler::new();

        let model = KeyboardState { handler };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {
        println!("received an update");
        let key_pressed = match _message {
            Msg::KeyPressed(k, _, _, _) => Some(k),
        };
        let cx = self.handler.get_context();
        cx.select_font_face(
            "Arial Black",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Bold,
        );

        cx.set_source_rgb(0.0, 0.0, 0.0);
        cx.set_font_size(18.0);
        let mut x = VSTART;
        let mut y = HSTART;
        let mut iter = QWERTY.iter();
        for (i, row) in LAYOUT.iter().enumerate() {
            let mut row = *row;
            while row > 0 {
                cx.set_source_rgb(0.0, 0.0, 0.0);
                row -= 1;
                if let Some((cell, size)) = iter.next() {
                    if let Some(key) = key_pressed {
                        if key
                            .name()
                            .is_some_and(|x| x.to_lowercase().as_str().eq_ignore_ascii_case(*cell))
                        {
                            cx.set_source_rgb(1.0, 0.0, 0.0);
                        }
                    }
                    cx.move_to(x, y);
                    cx.show_text(cell).expect("should display this char");
                    x += UNIT * size;
                }
            }
            x = VSTART;
            y += UNIT;
        }
    }
}

struct PracticeComp {
    practice: Practice,
    handler: DrawHandler,
}

#[relm4::component]
impl SimpleComponent for PracticeComp {
    type Init = Practice;
    type Input = Msg;
    type Output = ();
    view! {
            gtk::Box {
                #[local_ref]
                area -> gtk::DrawingArea {
                    set_vexpand: true,
                    set_hexpand: true,
                    inline_css: "border: 2px solid red",

                },
            }
    }
    fn init(
        practice: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        println!("Received an init");
        let handler = DrawHandler::new();

        let model = PracticeComp { practice, handler };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        println!("received an update");
        let key_pressed = match message {
            Msg::KeyPressed(k, _, _, _) => Some(k),
        };
        let cx = self.handler.get_context();
        cx.select_font_face(
            "Arial Black",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Bold,
        );

        cx.set_source_rgb(0.0, 0.0, 0.0);
        cx.set_font_size(18.0);
        let mut x = VSTART;
        let mut y = HSTART;
        for (i, w) in self.practice.iter().take(25).enumerate() {
            if i % 5 == 0 {
                x = VSTART;
                y += UNIT;
            }
            cx.move_to(x, y);
            cx.show_text(w.into_str()).expect("prints the word");
            x += 15.0 * w.len() as f64;
        }
    }
}

struct App {
    stats: Controller<Stats>,
    keyboard_state: Controller<KeyboardState>,
    practice_comp: Controller<PracticeComp>,
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
              my_practice -> gtk::Box {},
              #[local_ref]
              my_ks -> gtk::Box {},
            },
        },

    }

    fn init(
        practice: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats = Stats::builder().launch(Stats::new()).detach();
        let keyboard_state = KeyboardState::builder().launch(()).detach();
        let practice_comp = PracticeComp::builder().launch(practice).detach();
        let model = App {
            stats,
            keyboard_state,
            practice_comp,
        };
        let my_stats = model.stats.widget();
        let my_ks = model.keyboard_state.widget();
        let my_practice = model.practice_comp.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::KeyPressed(_, _, _, _) => {
                self.stats.emit(msg.clone());
                self.keyboard_state.emit(msg);
                self.practice_comp.emit(msg);
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
