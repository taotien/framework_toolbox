use std::process::Command;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, slider, text, toggler,
};
use iced::{alignment, window, Alignment, Color, Length, Sandbox, Settings};
use serde::Deserialize;

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

// ectool fwchargelimit
// ectool fan_duty
// ectool autofanctrl

#[derive(Deserialize)]
struct Toolbox {
    battery_limit: u8,
    fan_duty: u8,
    fan_auto: bool,
    // backlight_auto: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    BatteryLimitChanged(u8),
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    Apply,
    // BacklightAutoToggled { value: bool },
}

impl Sandbox for Toolbox {
    type Message = Message;

    fn new() -> Self {
        let config_exists = false;
        if config_exists {
            todo!()
        } else {
            Toolbox {
                battery_limit: 69,
                fan_duty: 42,
                fan_auto: true,
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
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
            }
            Message::Apply => {
                let batt_arg = String::from(format!(
                    "ectool fwchargelimit {}",
                    self.battery_limit.to_string()
                ));

                let fan_arg = String::from(if self.fan_auto {
                    "ectool autofanctrl".to_string()
                } else {
                    format!("ectool fanduty {}", self.fan_duty.to_string())
                });

                let sh_arg = format!("{}; {};", batt_arg, fan_arg);

                let _ = Command::new("pkexec")
                    .args(["sh", "-c"])
                    .arg(sh_arg)
                    .output()
                    .expect("failed to execute process");
            }
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
            button("Apply").on_press(Message::Apply),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center);

        container(content).center_x().into()
    }
}
