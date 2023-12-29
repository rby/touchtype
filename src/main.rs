use comp::keyboard::KeyboardState;
use comp::stats::StatsComp;
use gtk::prelude::*;
use rand::thread_rng;
use relm4::drawing::DrawHandler;
use relm4::{gtk::Inhibit, prelude::*};
use session::{Practice, TouchState};
use std::path::Path;
use std::time::Instant;

mod comp;
mod msg;
mod session;
mod stats;
mod utils;
use crate::msg::Msg;
use crate::session::Touch;
use crate::stats::Stats;
use crate::utils::{Clear, HasDrawHandler};

const UNIT: f64 = 30.0;
const WORDS_PER_LINE: usize = 5;
// TODO should be parsed from some resource files
const HSTART: f64 = 100.0;
const VSTART: f64 = 100.0;

#[derive(Debug)]
struct UpdateDrawingMsg;

struct PracticeComp {
    practice: Practice,
    handler: DrawHandler,
}

impl<'a> HasDrawHandler<'a> for PracticeComp {
    fn draw_handler_mut(&'a mut self) -> &'a mut DrawHandler {
        &mut self.handler
    }
}

impl PracticeComp {
    fn draw(&mut self, t: &Touch) {
        let cx = self.handler.get_context();

        // TODO we can also clear just one rectangle
        self.clear();
        cx.select_font_face(
            "Courier New",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Normal,
        );
        cx.set_source_rgb(0.0, 0.0, 0.0);
        cx.set_font_size(10.0);
        cx.move_to(10.0, 10.0);
        let debug_text = format!("{:?}", self.practice);
        cx.show_text(debug_text.as_str())
            .expect("display some debug");
        // the real text
        cx.select_font_face(
            "Arial Black",
            gtk::cairo::FontSlant::Normal,
            gtk::cairo::FontWeight::Bold,
        );
        cx.set_source_rgb(0.0, 0.0, 0.0);
        cx.set_font_size(18.0);

        let mut x = VSTART;
        let mut y = HSTART;
        let mut cw = 0;
        for (c, state, i) in self.practice.iter() {
            // reset x and go down every WORDS_PER_LINE words
            if i != cw && i % WORDS_PER_LINE == 0 {
                x = VSTART;
                y += UNIT;
            }
            if cw != i {
                cw += 1;
            }
            if c == Touch::Space {
                x += 7.0;
            }
            cx.move_to(x, y);
            // reset
            cx.set_source_rgb(0.0, 0.0, 0.0);
            // setup the color and any needed decoration that is function
            // of the state of the practice for each touch
            match state {
                TouchState::Next => {
                    // display an underline for the next char
                    cx.move_to(x, y + UNIT / 5.0);
                    cx.show_text("_").expect("underline");
                    cx.move_to(x, y);
                }
                TouchState::Current => {
                    if self.practice.check(&t) == Some(true) {
                        cx.set_source_rgb(0.0, 1.0, 0.0);
                    } else {
                        cx.set_source_rgb(1.0, 0.0, 0.0);
                    }
                }

                TouchState::Attempted(true) => {
                    cx.set_source_rgb(0.5, 0.5, 0.5);
                }
                TouchState::Attempted(false) => {
                    cx.set_source_rgb(0.8, 0.5, 0.5);
                }
                TouchState::Future => {}
            }
            // draws the char itself
            match c {
                Touch::Space => {
                    cx.show_text(".").expect("print the char");
                    x += cx.text_extents(".").unwrap().width();
                    x += 7.0;
                }
                Touch::Char(c) => {
                    let text = c.to_string();

                    cx.show_text(text.as_str()).expect("prints the char");
                    x += cx.text_extents(text.as_str()).unwrap().width() + char_adjust_width(c);
                }
            }
        }
    }
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
        let handler = DrawHandler::new();

        let model = PracticeComp { practice, handler };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            Msg::KeyPressed(_, t, _, _) => {
                self.draw(&t);
                self.practice.press(&t);
            }
        };
    }
}

/// Returns the space after the width of the char to adjust for
/// specific chars
fn char_adjust_width(c: char) -> f64 {
    match c {
        'i' | 'l' => 1.2,
        'w' | 'x' | 'y' => 0.8,
        _ => 1.0,
    }
}

struct App {
    stats: Controller<StatsComp>,
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
                connect_key_pressed[sender] => move |_, keyval, _, state| {
                    let now = Instant::now();
                    if let Some(touch) = keyval.to_unicode().map(Touch::from) {
                        sender.input(Msg::KeyPressed(keyval, touch, state, now));
                    }
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
        let stats = StatsComp::builder().launch(Stats::new()).detach();
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
                self.stats.emit(msg);
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
    let practice = Practice::generate(&mut rng, 25, Path::new("./data/t8.shakespeare.freq"))
        .expect("should load correctly");
    println!("next practice is : {practice:?}");
    app.run::<App>(practice);
}
