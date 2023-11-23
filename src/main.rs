use gtk::prelude::*;
use relm4::prelude::*;

#[derive(Default)]
struct App {}

#[relm4::component]
impl SimpleComponent for App {
    type Init = ();
    type Input = ();
    type Output = ();

    view! {
        main_window = gtk::ApplicationWindow {
            set_title: Some("Type Touching"),
            set_default_size: (640, 800),
        },
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {};
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}

fn main() {
    let app = RelmApp::new("TouchTyping Master");
    app.run::<App>(());
}
