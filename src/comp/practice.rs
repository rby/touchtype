use gtk::prelude::*;
use relm4::gtk;
use relm4::prelude::*;
use relm4::{drawing::DrawHandler, ComponentParts, ComponentSender, SimpleComponent};

use crate::model::Practice;
use crate::model::Touch;
use crate::model::TouchState;
use crate::msg::Msg;
use crate::run_enumerate::run_enumerate_with;
use crate::utils::{Clear, HasDrawHandler};

const UNIT: f64 = 30.0;
const WORDS_PER_LINE: usize = 5;
// TODO should be parsed from some resource files
const HSTART: f64 = 100.0;
const VSTART: f64 = 100.0;

pub(crate) struct PracticeComp {
    practice: Practice,
    handler: DrawHandler,
    saved: bool,
}

impl<'a> HasDrawHandler<'a> for PracticeComp {
    fn draw_handler_mut(&'a mut self) -> &'a mut DrawHandler {
        &mut self.handler
    }
}

impl PracticeComp {
    fn draw(&mut self, t: Option<Touch>) {
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
        for (w, (c, state, _)) in
            run_enumerate_with(&mut self.practice.iter(), |x| x.2 / WORDS_PER_LINE)
        {
            // if the current word has changed reset x and go down
            if w != cw {
                x = VSTART;
                y += UNIT;
            }
            cw = w;
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
                TouchState::Current(is_first) => {
                    if is_first {
                        cx.move_to(x, y + UNIT / 5.0);
                        cx.show_text("_").expect("underline");
                        cx.move_to(x, y);
                    }
                    if let Some(t) = t {
                        if self.practice.check(&t) == Some(true) {
                            cx.set_source_rgb(0.0, 1.0, 0.0);
                        } else {
                            cx.set_source_rgb(1.0, 0.0, 0.0);
                        }
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
                    x += cx.text_extents(".").unwrap().x_advance() + 7.0;
                }
                Touch::Char(c) => {
                    let text = c.to_string();

                    cx.show_text(text.as_str()).expect("prints the char");
                    x += cx.text_extents(text.as_str()).unwrap().x_advance(); // + char_adjust_width(c);
                }
            }
        }
    }
}

#[relm4::component(pub)]
impl SimpleComponent for PracticeComp {
    type Init = Practice;
    type Input = Msg;
    type Output = Msg;
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

        let model = PracticeComp {
            practice,
            handler,
            saved: false,
        };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            Msg::KeyPressed(_, t, _, _) if !self.saved => {
                self.draw(Some(t));
                if self.practice.press(&t).is_none() {
                    let p = self.practice.clone();
                    println!("practice saved to {}", p.name());
                    sender
                        .output(Msg::PracticeEnd(p))
                        .expect("should output End event");
                    self.saved = true;
                }
            }
            Msg::PracticeStart(practice) => {
                println!("[PracticeComp] received a new practice");
                self.saved = false;
                self.practice = practice;
                self.draw(None);
            }
            _ => (),
        };
    }
}
