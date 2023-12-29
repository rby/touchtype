use relm4::gtk::gdk::{Key, ModifierType};
use std::time::Instant;

use crate::session::Touch;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Msg {
    KeyPressed(Key, Touch, ModifierType, Instant),
}
