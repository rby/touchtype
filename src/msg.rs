/// Messages and events shared between components
use relm4::gtk::gdk::{Key, ModifierType};
use std::time::Instant;

use crate::model::{Practice, Touch};

#[derive(Debug, Clone)]
pub(crate) enum Msg {
    KeyPressed(Key, Touch, ModifierType, Instant),
    PracticeEnd(Practice),
    PracticeStart(Practice),
}
