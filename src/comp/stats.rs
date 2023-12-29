use relm4::gtk;
use relm4::ComponentParts;
use relm4::ComponentSender;
use relm4::SimpleComponent;

use crate::{msg::Msg, stats::Stats};

// puts Stats comp implem here
pub(crate) struct StatsComp(Stats);

#[relm4::component(pub)]
impl SimpleComponent for StatsComp {
    type Init = Stats;
    type Input = Msg;
    type Output = ();

    view! {
        gtk::Label {
            #[watch]
            set_label: &format!("{}/s", model.0.avg_key_s())
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = StatsComp(init);
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            Msg::KeyPressed(_, _, _, _) => self.0.add(msg),
        }
    }
}
