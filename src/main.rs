use comp::keyboard::KeyboardState;
use comp::practice::PracticeComp;
use comp::stats::StatsComp;
use gtk::prelude::*;
use rand::thread_rng;
use relm4::{gtk::Inhibit, prelude::*};
use session::Practice;
use std::convert::identity;
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

#[derive(Debug)]
struct UpdateDrawingMsg;

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
        let practice_comp = PracticeComp::builder()
            .launch(practice)
            .forward(sender.input_sender(), identity);
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
                self.keyboard_state.emit(msg.clone());
                self.practice_comp.emit(msg.clone());
            }
            Msg::PracticeEnd(practice) => {
                let home = env!("HOME");
                let path = Path::new(home).join(".config/touchtype");
                practice
                    .save(path.as_path())
                    .expect("practice should be saved");
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
