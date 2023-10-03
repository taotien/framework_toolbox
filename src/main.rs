use clap::Parser;
use iced::{Application, Settings};

use framework_toolbox::{Toolbox, ToolboxFlags};

pub fn main() -> iced::Result {
    let args = ToolboxFlags::parse();

    Toolbox::run(Settings {
        exit_on_close_request: false,
        window: iced::window::Settings {
            size: (400, 500),
            resizable: false,
            ..iced::window::Settings::default()
        },
        flags: args,
        ..Settings::default()
    })
}
