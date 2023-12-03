use std::time::{Duration, Instant};

use gtk::prelude::*;
use relm4::{
    gtk::{
        gdk::{Key, ModifierType},
        Inhibit,
    },
    prelude::*,
};

struct Stats {
    duration_sum: Duration,
    last_key: Option<Instant>,
    count: u32,
}

impl Stats {
    fn add(&mut self, msg: Msg) {
        match msg {
            Msg::KeyPressed(_, _, _, ts) => {
                self.count += 1;
                self.duration_sum += match self.last_key {
                    Some(last_key) => ts.duration_since(last_key),
                    None => Duration::ZERO,
                };
                self.last_key = Some(ts);
            }
        }
    }

    fn avg_key_s(&self) -> f32 {
        if self.duration_sum.is_zero() {
            0.0
        } else {
            self.count as f32 / self.duration_sum.as_secs_f32()
        }
    }
    fn new() -> Self {
        Stats {
            duration_sum: Duration::ZERO,
            last_key: None,
            count: 0,
        }
    }
}

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
enum Msg {
    KeyPressed(Key, u32, ModifierType, Instant),
}

struct App {
    stats: Controller<Stats>,
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
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
            },
        },

    }

    fn init(
        _: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let stats = Stats::builder().launch(Stats::new()).detach();
        let model = App { stats };
        let my_stats = model.stats.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::KeyPressed(_, _, _, _) => {
                self.stats.emit(msg);
            }
        }
    }
}

fn main() {
    let app = RelmApp::new("TouchTyping Master");
    app.run::<App>(());
}
