use framework_toolbox::Toolbox;
use iced::{Application, Settings, Size};

fn main() -> iced::Result {
    Toolbox::run(Settings {
        window: iced::window::Settings {
            size: Size::new(400.0, 470.0),
            resizable: false,
            ..iced::window::Settings::default()
        },
        ..Settings::default()
    })
}
