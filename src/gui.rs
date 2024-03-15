use iced::{
    alignment, executor,
    widget::{
        button, column, container, horizontal_rule, horizontal_space, pick_list, row, slider, text,
        toggler,
    },
    Alignment, Application, Color, Element, Length, Theme,
};

use crate::{LedColor, Toolbox};

#[derive(Debug, Clone)]
pub enum Message {
    BatteryLimitChanged(u8),
    BatteryOneShot,
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    LedPowerSelected(LedColor),
    LedLeftSelected(LedColor),
    LedRightSelected(LedColor),
}

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
    };
}

macro_rules! width_align_spacing_map {
    ($($x:expr), *) => {
        $($x
            .width(Length::Fill)
            .align_items(Alignment::Center)
            // .spacing(10)
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
        match message {
            Message::BatteryLimitChanged(value) => {
                self.battery_limit = value;
            }
            Message::BatteryOneShot => {}
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
        }
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message, Self::Theme, iced::Renderer> {
        let space = 10;

        let title = column![
            text("Framework Toolbox")
                .width(Length::Fill)
                .size(42)
                .style(Color::from_rgb(0.5, 0.5, 0.5))
                .horizontal_alignment(alignment::Horizontal::Center),
            horizontal_rule(5),
            horizontal_space(),
        ];

        let battery_controls = column![
            text(format!("Battery Limit: {}%", self.battery_limit)),
            slider_block![
                text("40%"),
                slider(40..=100, self.battery_limit, Message::BatteryLimitChanged),
                text("100%")
            ],
            row![
                text("Charge to 100% once:").width(Length::Fill),
                button("100%").on_press(Message::BatteryOneShot)
            ]
        ];

        let fan_controls = column![
            text(format!("Fan Duty: {}", {
                if self.fan_auto {
                    "Auto"
                } else {
                    stringify!("{}%", self.fan_duty)
                }
            })),
            slider_block![
                text("0%"),
                slider(0..=100, self.fan_duty, Message::FanDutyChanged),
                text("100%")
            ],
            toggler("Auto".to_string(), self.fan_auto, Message::FanAutoToggled)
                .text_alignment(alignment::Horizontal::Right)
        ];

        let led_controls = width_align_spacing_map!(row![
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
        ]);

        let content: Element<_> = column![title, battery_controls, fan_controls, led_controls]
            .spacing(space)
            .padding(space)
            .align_items(Alignment::Center)
            .into();

        container(content).center_x().into()
    }
}
