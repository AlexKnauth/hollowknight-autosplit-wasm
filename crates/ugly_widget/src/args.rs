
pub use ugly_widget_derive::SetHeadingLevel;

use asr::settings::gui::{BoolArgs, TitleArgs};

// --------------------------------------------------------

pub trait SetHeadingLevel {
    fn set_heading_level(&mut self, heading_level: u32);
}

impl SetHeadingLevel for TitleArgs {
    fn set_heading_level(&mut self, heading_level: u32) {
        self.heading_level = heading_level;
    }
}

impl SetHeadingLevel for BoolArgs {
    fn set_heading_level(&mut self, _: u32) {
        ()
    }
}

impl SetHeadingLevel for () {
    fn set_heading_level(&mut self, _: u32) {
        ()
    }
}
