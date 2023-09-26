use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::Command;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, slider, text,
    toggler,
};
use iced::{
    alignment, executor, Alignment, Application, Color, Element, Length, Subscription, Theme,
};
use iced_native::{window, Event};

use crate::{LedColor, Toolbox};

#[derive(Clone, Debug)]
pub enum Message {
    Event(Event),
    BatteryLimitChanged(u8),
    BatteryOneShot,
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    BacklightAutoToggled(bool),
    LEDPowerSelected(LedColor),
    LEDLeftSelected(LedColor),
    LEDRightSelected(LedColor),
    Save,
}

impl Application for Toolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn title(&self) -> String {
        String::from("Framework Toolbox")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn new(_flags: ()) -> (Toolbox, iced::Command<Message>) {
        // check for existing config, otherwise default
        let mut tb = Toolbox::parse().unwrap_or_default();
        if tb.backlight_auto {
            tb.backlight_daemon = Some(
                Command::new("fwtb-ab")
                    .spawn()
                    .expect("couldn't start autobacklight"),
            )
        }

        (tb, iced::Command::none())
    }

    // TODO remove this
    fn subscription(&self) -> Subscription<Message> {
        let subs = vec![
            // dunno why no closure here
            iced_native::subscription::events().map(Message::Event),
        ];
        iced_native::Subscription::batch(subs)
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
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
            Message::BacklightAutoToggled(value) => {
                self.backlight_auto = value;
                if self.backlight_auto {
                    self.backlight_daemon = Some(
                        Command::new("fwtb-ab")
                            .spawn()
                            .expect("couldn't start autobacklight"),
                    )
                } else if let Some(c) = &mut self.backlight_daemon {
                    c.kill().expect("couldn't kill autobacklight");
                }
            }
            Message::LEDPowerSelected(value) => {
                self.led_power = Some(value);
            }
            Message::LEDLeftSelected(value) => {
                self.led_left = Some(value);
            }
            Message::LEDRightSelected(value) => {
                self.led_right = Some(value);
            }
            Message::Save => {
                let toml = toml_edit::easy::to_string(&self).unwrap();
                let mut conf = dirs::config_dir().unwrap();
                conf.push("fwtb.toml");
                let mut f = File::create(conf).unwrap();
                f.write_all("# Generated file, DO NOT EDIT!\n".as_bytes())
                    .unwrap();
                f.write_all(toml.as_bytes()).unwrap();
            }
            Message::Event(event) => {
                if let Event::Window(window::Event::CloseRequested) = event {
                    if let Some(c) = &mut self.backlight_daemon {
                        c.kill().expect("couldn't kill autobacklight");
                    }
                    self.should_exit = true;
                }
            }
        };
        iced::Command::none()
    }

    fn view(&self) -> iced::Element<Message> {
        let title = text("Framework Toolbox")
            .width(Length::Fill)
            .size(42)
            .style(Color::from([0.5, 0.5, 0.5]))
            .horizontal_alignment(alignment::Horizontal::Center);

        let space = 10;

        // Battery stuff
        //
        let battery_limit_slider =
            slider(40..=100, self.battery_limit, Message::BatteryLimitChanged);

        let battery_limit_row = row![
            text("40%")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Right),
            battery_limit_slider.width(Length::FillPortion(5)),
            text("100%")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left),
        ]
        .spacing(10);

        let battery_oneshot_row = row![
            text("Charge to 100% until unplugged:")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Right),
            button("100%").on_press(Message::BatteryOneShot),
        ]
        .align_items(Alignment::Center)
        .spacing(space)
        .padding(space);

        let battery_controls = column![
            text(format!("Battery Limit: {}%", self.battery_limit)),
            battery_limit_row,
            battery_oneshot_row
        ]
        .align_items(Alignment::Center)
        .spacing(space);

        // Fan stuff
        //
        let fan_duty_slider = slider(0..=100, self.fan_duty, Message::FanDutyChanged);

        let fan_duty_row = row![
            text("0%")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Right),
            fan_duty_slider.width(Length::FillPortion(5)),
            text("100%")
                .width(Length::Fill)
                .horizontal_alignment(alignment::Horizontal::Left)
        ]
        .spacing(space);

        let fan_auto_toggler =
            toggler(String::from("Auto"), self.fan_auto, Message::FanAutoToggled)
                .text_alignment(alignment::Horizontal::Right)
                .spacing(space);

        let fan_controls = column![
            text(format!("Fan Duty: {}", {
                if self.fan_auto {
                    "Auto".to_string()
                } else {
                    format!("{}%", self.fan_duty)
                }
            })),
            fan_duty_row,
            fan_auto_toggler
        ]
        .align_items(Alignment::Center)
        .spacing(space);

        // Backlight stuff
        //
        let backlight_auto_toggler = toggler(
            String::from("Auto"),
            self.backlight_auto,
            Message::BacklightAutoToggled,
        )
        .text_alignment(alignment::Horizontal::Right)
        .spacing(space);

        let backlight_controls = column![
            text(format!("Backlight: {}", {
                if self.backlight_auto {
                    "Auto".to_string()
                } else {
                    "Manual".to_string()
                }
            })),
            backlight_auto_toggler,
        ]
        .align_items(Alignment::Center)
        .spacing(space);

        // LED space
        //
        let led_left_picker =
            pick_list(&LedColor::ALL[..], self.led_left, Message::LEDLeftSelected);

        let led_power_picker = pick_list(
            &LedColor::ALL[..],
            self.led_power,
            Message::LEDPowerSelected,
        );

        let led_right_picker = pick_list(
            &LedColor::ALL[..],
            self.led_right,
            Message::LEDRightSelected,
        );

        let led_row = row![
            column![text("Left"), led_left_picker,]
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(space),
            column![text("Power"), led_power_picker,]
                .width(Length::Fill)
                .align_items(Alignment::Center)
                .spacing(space),
            column![text("Right"), led_right_picker,]
                .align_items(Alignment::Center)
                .width(Length::Fill)
                .spacing(space),
        ]
        .spacing(space);

        let led_controls = column![text("LED Colors"), led_row,]
            .align_items(Alignment::Center)
            .spacing(space);

        // Everything stuff
        //
        let content: Element<_> = column![
            title,
            horizontal_rule(5),
            horizontal_space(Length::Fill),
            battery_controls,
            fan_controls,
            backlight_controls,
            led_controls,
            button("Apply").on_press(Message::Save),
        ]
        .spacing(space)
        .padding(space)
        .align_items(Alignment::Center)
        .into();

        // container(content.explain(Color::BLACK)).center_x().into()
        container(content).center_x().into()
    }
}
