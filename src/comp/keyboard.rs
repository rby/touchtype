use gtk::prelude::*;
use relm4::gtk;
use relm4::gtk::gdk::Key;
use relm4::prelude::*;
use relm4::{drawing::DrawHandler, ComponentParts, ComponentSender, SimpleComponent};

use crate::msg::Msg;

const UNIT: f64 = 30.0;
// TODO should be parsed from some resource files
const QWERTY: &'static [(&str, f64)] = &[
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
const LAYOUT: &'static [usize] = &[14, 14, 13, 12, 1];
const HSTART: f64 = 100.0;
const VSTART: f64 = 100.0;

pub(crate) struct KeyboardComp {
    handler: DrawHandler,
}

impl KeyboardComp {
    fn draw(&mut self, k: Option<Key>) {
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
        for row in LAYOUT {
            for _ in 0..*row {
                cx.set_source_rgb(0.0, 0.0, 0.0);
                if let Some((cell, size)) = iter.next() {
                    match k.and_then(|k| k.name()) {
                        Some(x) if x.eq_ignore_ascii_case(*cell) => {
                            cx.set_source_rgb(0.0, 1.0, 0.0)
                        }
                        _ => (),
                    };
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

#[relm4::component(pub)]
impl SimpleComponent for KeyboardComp {
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
        let handler = DrawHandler::new();

        let model = KeyboardComp { handler };
        let area = model.handler.drawing_area();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, _message: Self::Input, _sender: ComponentSender<Self>) {
        match _message {
            Msg::KeyPressed(k, _, _, _) => self.draw(Some(k)),
            _ => self.draw(None),
        };
    }
}
