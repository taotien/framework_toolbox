use anyhow::Result;
use iced::{
    alignment, executor,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, pick_list, row, slider, text,
        toggler, Space,
    },
    Alignment, Application, Color, Element, Length, Theme,
};

use crate::{
    ectool::{Ectool, LedSide},
    LedColor, Toolbox,
};

#[derive(Debug, Clone)]
pub enum Message {
    BatteryLimitChanged(u8),
    BatteryOneShot,
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    LedPowerSelected(LedColor),
    LedLeftSelected(LedColor),
    LedRightSelected(LedColor),
    // Applied,
    Apply,
}

const SPACE: u16 = 10;

macro_rules! slider_block {
    ($left:expr, $middle:expr, $right:expr) => {
        row![
            $left
                .horizontal_alignment(alignment::Horizontal::Right)
                .width(Length::Fill),
            $middle.width(Length::FillPortion(5)),
            $right
                .horizontal_alignment(alignment::Horizontal::Left)
                .width(Length::Fill),
        ]
        .spacing(SPACE)
    };
}

macro_rules! width_align_spacing_map {
    ($($x:expr), *) => {
        $($x
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .spacing(SPACE)
        ), *
    };
}

impl Application for Toolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn title(&self) -> String {
        "Framework Toolbox".to_string()
    }

    fn theme(&self) -> Self::Theme {
        Theme::Dark
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let tb = Toolbox::parse().unwrap_or_default();

        (tb, iced::Command::none())
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        self.applied = false;
        match message {
            Message::BatteryLimitChanged(value) => {
                self.battery_limit = value;
                self.battery_oneshot_applied = false;
            }
            Message::BatteryOneShot => {
                match Ectool::FwChargeOnce(100).call() {
                    Ok(()) => self.battery_oneshot_applied = true,
                    Err(_) => {
                        self.error = true;
                    }
                };
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
            }
            Message::LedPowerSelected(value) => {
                self.led_power = Some(value);
            }
            Message::LedLeftSelected(value) => {
                self.led_left = Some(value);
            }
            Message::LedRightSelected(value) => {
                self.led_right = Some(value);
            }
            // Message::Applied => {
            //     self.applied = true;
            // }
            Message::Apply => {
                // lmao this is so cursed
                let result = || -> Result<()> {
                    Ectool::FwChargeLimit(self.battery_limit).call()?;
                    if self.fan_auto {
                        Ectool::AutoFanCtrl.call()?;
                    } else {
                        Ectool::FanDuty(self.fan_duty).call()?;
                    }
                    if let Some(color) = self.led_power {
                        Ectool::Led(LedSide::Power(color)).call()?;
                    }
                    if let Some(color) = self.led_left {
                        Ectool::Led(LedSide::Left(color)).call()?;
                    }
                    if let Some(color) = self.led_right {
                        Ectool::Led(LedSide::Right(color)).call()?;
                    }
                    Ok(())
                };
                match result() {
                    Ok(()) => self.applied = true,
                    Err(_) => self.error = true,
                }
                // return iced::Command::perform(apply(self.clone()), |_| Message::Applied);
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let title = text("Framework Toolbox")
            .width(Length::Fill)
            .size(35)
            .style(Color::from_rgb(0.5, 0.5, 0.5))
            .horizontal_alignment(alignment::Horizontal::Center);

        let battery_controls = column![
            text(format!("Battery Limit: {}%", self.battery_limit)),
            slider_block![
                text("40%"),
                slider(40..=100, self.battery_limit, Message::BatteryLimitChanged),
                text("100%")
            ],
            row![
                Space::with_width(Length::Fill),
                button(if self.battery_oneshot_applied {
                    "Applied!"
                } else {
                    "100% once"
                })
                .on_press(Message::BatteryOneShot),
            ]
            .padding(SPACE)
        ]
        .align_items(Alignment::Center)
        .spacing(SPACE);

        let fan_controls = column![
            text(format!("Fan Duty: {}", {
                if self.fan_auto {
                    "Auto".to_string()
                } else {
                    format!("{}%", self.fan_duty)
                }
            })),
            slider_block![
                text("0%"),
                slider(0..=100, self.fan_duty, Message::FanDutyChanged),
                text("100%")
            ],
            row![
                Space::with_width(Length::Fill),
                toggler("Auto".to_string(), self.fan_auto, Message::FanAutoToggled)
                    .width(Length::Shrink)
                    .spacing(SPACE)
            ]
            .padding(SPACE)
        ]
        .align_items(Alignment::Center)
        .spacing(SPACE);

        let led_controls = column![
            text("LED Colors"),
            width_align_spacing_map!(row![
                column![
                    text("Left"),
                    pick_list(&LedColor::ALL[..], self.led_left, Message::LedLeftSelected)
                ],
                column![
                    text("Power"),
                    pick_list(
                        &LedColor::ALL[..],
                        self.led_power,
                        Message::LedPowerSelected
                    )
                ],
                column![
                    text("Right"),
                    pick_list(
                        &LedColor::ALL[..],
                        self.led_right,
                        Message::LedRightSelected
                    )
                ]
            ])
            .width(Length::Shrink)
        ]
        .align_items(Alignment::Center);

        let content: Element<_> = column![
            title,
            horizontal_rule(5),
            horizontal_space(),
            battery_controls,
            fan_controls,
            led_controls,
            horizontal_space(),
            button(if self.applied {
                "Applied!"
            } else if self.error {
                "Error!"
            } else {
                "Apply"
            })
            .on_press(Message::Apply)
        ]
        .align_items(Alignment::Center)
        .spacing(SPACE)
        .padding(SPACE)
        .into();

        container(content).center_x().into()
    }
}

// async fn apply(tb: Toolbox) {
//     if let Some(prev) = tb.prev {

//     }
// }
