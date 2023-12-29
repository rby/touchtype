use relm4::gtk::gdk::{Key, ModifierType};
use std::time::Instant;

use crate::session::{Practice, Touch};

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    KeyPressed(Key, Touch, ModifierType, Instant),
    PracticeEnd(Practice),
}
