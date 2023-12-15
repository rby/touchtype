use relm4::gtk::gdk::{Key, ModifierType};
use std::time::Instant;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Msg {
    KeyPressed(Key, u32, ModifierType, Instant),
}
