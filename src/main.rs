use comp::keyboard::KeyboardComp;
use comp::practice::PracticeComp;
use comp::stats::StatsComp;
use gtk::prelude::*;
use model::{Practice, PracticeGenerator};
use rand::rngs::ThreadRng;
use rand::thread_rng;
use relm4::tokio;
use relm4::{gtk::Inhibit, prelude::*};
use std::convert::identity;
use std::path::Path;
use std::time::{Duration, Instant};

mod comp;
mod model;
mod msg;
mod stats;
mod utils;
mod uniq_enumerate;
use crate::model::Touch;
use crate::msg::Msg;
use crate::stats::Stats;

#[derive(Debug)]
struct UpdateDrawingMsg;

struct App {
    stats: Controller<StatsComp>,
    keyboard_state: Controller<KeyboardComp>,
    practice_comp: Controller<PracticeComp>,
    practice_generator: PracticeGenerator<ThreadRng>,
}

#[relm4::component]
impl Component for App {
    type Init = (Practice, PracticeGenerator<ThreadRng>);
    type Input = Msg;
    type Output = Msg;
    type CommandOutput = Msg;

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
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (practice, practice_generator) = init;
        let stats = StatsComp::builder().launch(Stats::new()).detach();
        let keyboard_state = KeyboardComp::builder().launch(()).detach();
        let msg = Msg::PracticeStart(practice.clone());
        let practice_comp = PracticeComp::builder()
            .launch(practice)
            .forward(sender.input_sender(), identity);
        let model = App {
            stats,
            keyboard_state,
            practice_comp,
            practice_generator,
        };
        let my_stats = model.stats.widget();
        let my_ks = model.keyboard_state.widget();
        let my_practice = model.practice_comp.widget();
        let widgets = view_output!();
        sender.command(|out, shutdown| {
            shutdown.register(async move{
                tokio::time::sleep(Duration::from_millis(200)).await;
                out.send(msg).unwrap()
            })
            .drop_on_shutdown()
        });

        ComponentParts { model, widgets }
    }
    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match msg {
            Msg::KeyPressed(_, _, _, _) | Msg::PracticeStart(_) => {
                println!("emitting {:?}", msg);
                self.practice_comp.emit(msg.clone());
                self.stats.emit(msg.clone());
                self.keyboard_state.emit(msg.clone());
            }
            Msg::PracticeEnd(practice) => {
                let home = env!("HOME");
                let path = Path::new(home).join(".config/touchtype");
                practice
                    .save(path.as_path())
                    .expect("practice should be saved");
                let practice = self
                    .practice_generator
                    .generate()
                    .expect("generate a new practice");
                sender.input(Msg::PracticeStart(practice));
            }
        }
    }
    fn update_cmd(
            &mut self,
            message: Self::CommandOutput,
            sender: ComponentSender<Self>,
            root: &Self::Root,
        ) {
        self.update(message, sender, root)
    }
}

// TODO should be a result later
fn main() {
    let app = RelmApp::new("TouchTyping Master");
    let rng = thread_rng();
    let mut practice_generator =
        PracticeGenerator::<ThreadRng>::new(rng, 25, "./data/t8.shakespeare.freq");
    let practice = practice_generator
        .generate()
        .expect("should generate first practice");
    app.run::<App>((practice, practice_generator));
}
