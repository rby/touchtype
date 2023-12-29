use relm4::drawing::DrawHandler;
use relm4::gtk;

pub trait Clear<'a> {
    fn clear(&'a mut self);
}

pub trait HasDrawHandler<'a> {
    fn draw_handler_mut(&'a mut self) -> &'a mut DrawHandler;
}

impl<'a, T: HasDrawHandler<'a>> Clear<'a> for T {
    fn clear(&'a mut self) {
        let handler = self.draw_handler_mut();
        let cx = handler.get_context();

        let op = cx.operator();
        cx.set_operator(gtk::cairo::Operator::Clear);
        cx.paint().expect("should paint");
        cx.set_operator(op);
    }
}
