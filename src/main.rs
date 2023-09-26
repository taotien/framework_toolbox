use iced::{Application, Settings};

use framework_toolbox::Toolbox;

pub fn main() -> iced::Result {
    Toolbox::run(Settings {
        exit_on_close_request: false,
        window: iced::window::Settings {
            size: (400, 500),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
