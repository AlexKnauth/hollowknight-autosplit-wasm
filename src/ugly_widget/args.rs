
use asr::settings::gui::{BoolArgs, TitleArgs};

// --------------------------------------------------------

pub trait SetHeadingLevel {
    fn set_heading_level(&mut self, heading_level: u32);
}

#[macro_export]
macro_rules! impl_SetHeadingLevel_for {
    ( $x:ty ) => {
        impl SetHeadingLevel for $x {
            fn set_heading_level(&mut self, heading_level: u32) {
                self.heading_level = heading_level;
            }
        }
    }
}

impl_SetHeadingLevel_for!(TitleArgs);

impl SetHeadingLevel for BoolArgs {
    fn set_heading_level(&mut self, _: u32) {
        ()
    }
}
