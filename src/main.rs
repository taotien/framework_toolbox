use std::io::Write;
use std::process::{ChildStdin, Command, Stdio};

use iced::widget::{
    column, container, horizontal_rule, horizontal_space, row, slider, text, toggler,
};
use iced::{alignment, window, Alignment, Color, Length, Sandbox, Settings};

pub fn main() -> iced::Result {
    Toolbox::run(Settings {
        window: iced::window::Settings {
            size: (400, 800),
            resizable: false,
            ..window::Settings::default()
        },
        ..Settings::default()
    })
}

struct Toolbox {
    battery_limit: u8,
    fan_duty: u8,
    fan_auto: bool,
    backlight: u32,
    backlight_auto: bool,
    daemon: ChildStdin,
}

#[derive(Clone, Debug)]
pub enum Message {
    BatteryLimitChanged(u8),
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    BacklightChanged(u32),
    BacklightAutoToggled(bool),
    // Apply,
}

impl Sandbox for Toolbox {
    type Message = Message;

    fn new() -> Self {
        let config_exists = false;
        let mut daemon = Command::new("pkexec")
            .arg("/home/tao/Projects/framework_toolbox/target/debug/daemon")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to open daemon");
        let daemon_stdin = daemon.stdin.take().expect("couldn't take stdin of daemon");

        if config_exists {
            todo!()
        } else {
            Toolbox {
                battery_limit: 69,
                fan_duty: 42,
                fan_auto: true,
                backlight: 48000,
                backlight_auto: true,
                daemon: daemon_stdin,
            }
        }
    }

    fn title(&self) -> String {
        String::from("Framework Toolbox")
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::BatteryLimitChanged(value) => {
                self.battery_limit = value;
                write!(self.daemon, "charge\n{}\n", self.battery_limit)
                    .expect("couldn't write to daemon");
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
                write!(self.daemon, "fan\n{}\n", self.fan_duty).expect("couldn't write to daemon");
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
                if value == false {
                    write!(self.daemon, "fan\n{}\n", self.fan_duty)
                        .expect("couldn't write to daemon");
                } else {
                    write!(self.daemon, "autofan\n").expect("couldn't write to daemon");
                }
            }
            Message::BacklightChanged(value) => {
                self.backlight = value;
                self.backlight_auto = false;
                write!(self.daemon, "backlight\n{}\n", value).expect("couldn't write to daemon");
            }
            Message::BacklightAutoToggled(value) => {
                self.backlight_auto = value;
            } // Message::Apply => {
              //     let mut stdin = self.daemon_handle.stdin.take().expect("couldn't take stdin of daemon");
              //     stdin.write_all();
              // }
        }
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let title = text("Framework Toolbox")
            .width(Length::Fill)
            .size(42)
            .style(Color::from([0.5, 0.5, 0.5]))
            .horizontal_alignment(alignment::Horizontal::Center);

        // Battery stuff
        //
        let battery_limit_slider =
            slider(40..=100, self.battery_limit, Message::BatteryLimitChanged)
                .width(Length::Units(200));

        let battery_limit_row = row![text("40%"), battery_limit_slider, text("100%")]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::End);

        // Fan stuff
        //
        let fan_duty_slider =
            slider(0..=100, self.fan_duty, Message::FanDutyChanged).width(Length::Units(200));

        let fan_duty_row = row![text("0%"), fan_duty_slider, text("100%")]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::End);

        let fan_auto_toggler =
            toggler(String::from("Auto"), self.fan_auto, Message::FanAutoToggled)
                .text_alignment(alignment::Horizontal::Right)
                .width(Length::Shrink)
                .spacing(5);

        let fan_controls = column![fan_duty_row, fan_auto_toggler].align_items(Alignment::End);

        // Backlight stuff
        //
        let backlight_slider = slider(1000..=96000, self.backlight, Message::BacklightChanged)
            .width(Length::Units(200));

        let backlight_row = row![text("1000"), backlight_slider, text("96000")]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::End);

        let backlight_auto_toggler = toggler(
            String::from("Auto"),
            self.backlight_auto,
            Message::BacklightAutoToggled,
        )
        .text_alignment(alignment::Horizontal::Right)
        .width(Length::Shrink)
        .spacing(5);

        let backlight_controls =
            column![backlight_row, backlight_auto_toggler].align_items(Alignment::End);

        // Everything stuff
        //
        let content = column![
            title,
            horizontal_rule(5),
            horizontal_space(Length::Fill),
            text(format!("Battery Limit: {}%", self.battery_limit)),
            battery_limit_row,
            text(format!("Fan Duty: {}", {
                if self.fan_auto {
                    "Auto".to_string()
                } else {
                    format!("{}%", self.fan_duty)
                }
            })),
            fan_controls,
            text(format!("Backlight: {}", {
                if self.backlight_auto {
                    "Auto".to_string()
                } else {
                    format!("{}", self.backlight)
                }
            })),
            backlight_controls,
            // button("Apply").on_press(Message::Apply),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center);

        container(content).center_x().into()
    }
}
