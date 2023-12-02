use std::time::{Duration, Instant};

use gtk::prelude::*;
use relm4::{
    channel,
    gtk::{
        gdk::{Key, ModifierType},
        Inhibit,
    },
    prelude::*,
    Sender,
};

#[derive(Default)]
struct App {}

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

#[derive(Debug)]
enum Msg {
    KeyPressed(Key, u32, ModifierType, Instant),
}

#[relm4::component]
impl SimpleComponent for App {
    type Init = Sender<Msg>;
    type Input = ();
    type Output = ();

    view! {
          main_window = gtk::ApplicationWindow {
              set_title: Some("Type Touching"),
              set_default_size: (640, 800),
              add_controller = gtk::EventControllerKey {
                  /*
                   * guint keyval,
    guint keycode,
    GdkModifierType state,
    gpointer user_data
                   */
                  connect_key_pressed => move |_, keyval, keycode, state| {
                      let now = Instant::now();
                      let msg = Msg::KeyPressed(keyval, keycode, state, now);
                      let _ = init.send(msg);
                      Inhibit(false)

                  }
              },
          },

      }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let (tx, rx) = channel::<Msg>();
    let mut stats = Stats::new();
    let _backend = relm4::spawn(async move {
        loop {
            match rx.recv().await.expect("should be a msg") {
                msg @ Msg::KeyPressed(k, _, modifier, time) => {
                    stats.add(msg);
                    println!("< key press {k} and modifier {modifier} as {time:?}");
                    println!("avg key/s : {}", stats.avg_key_s())
                }
            }
        }
    });
    let app = RelmApp::new("TouchTyping Master");
    app.run::<App>(tx);
}
