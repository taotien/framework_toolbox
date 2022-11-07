use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

use iced::widget::{
    column, container, horizontal_rule, horizontal_space, row, slider, text, toggler,
};
use iced::{Subscription, alignment, executor, Alignment, Application, Color, Length, Settings, Theme};

use iced_native::{window, Event};

pub fn main() -> iced::Result {
    Toolbox::run(Settings {
        exit_on_close_request: false,
        window: iced::window::Settings {
            size: (400, 800),
            resizable: false,
            ..iced::window::Settings::default()
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
    backlight_daemon: Option<Child>,
    daemon: ChildStdin,
    should_exit: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    Event(Event),
    BatteryLimitChanged(u8),
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    BacklightChanged(u32),
    BacklightAutoToggled(bool),
    // Apply,
}

impl Application for Toolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Toolbox, iced::Command<Message>) {
        let config_exists = false;
        // println!("running in {}", std::env::current_dir().unwrap().display());
        let mut daemon = Command::new("pkexec")
            .arg("fwtbd")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to open daemon");
        let daemon_stdin = daemon.stdin.take().expect("couldn't take stdin of daemon");

        // writeln!(&daemon_stdin, "charge\n{}", 69).expect("couldn't write to daemon");
        // writeln!(&daemon_stdin, "autofan\n").expect("couldn't write to daemon");

        if config_exists {
            todo!()
        } else {
            (
                Toolbox {
                    battery_limit: 69,
                    fan_duty: 42,
                    fan_auto: true,
                    backlight: 48000,
                    backlight_auto: true,
                    backlight_daemon: None,
                    daemon: daemon_stdin,
                    should_exit: false,
                },
                iced::Command::none(),
            )
        }
    }

    fn title(&self) -> String {
        String::from("Framework Toolbox")
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::BatteryLimitChanged(value) => {
                self.battery_limit = value;
                writeln!(&self.daemon, "charge\n{}", self.battery_limit)
                    .expect("couldn't write to daemon");
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
                writeln!(&self.daemon, "fan\n{}", self.fan_duty).expect("couldn't write to daemon");
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
                if !value {
                    writeln!(&self.daemon, "fan\n{}", self.fan_duty)
                        .expect("couldn't write to daemon");
                } else {
                    writeln!(&self.daemon, "autofan").expect("couldn't write to daemon");
                }
            }
            Message::BacklightChanged(value) => {
                self.backlight = value;
                writeln!(&self.daemon, "backlight\n{}", value).expect("couldn't write to daemon");
            }
            Message::BacklightAutoToggled(value) => {
                self.backlight_auto = value;
                if self.backlight_auto {
                    self.backlight_daemon = Some(
                        Command::new("fwtb-ab")
                            .spawn()
                            .expect("couldn't start autobacklight"),
                    )
                } else {
                    if let Some(c) = &mut self.backlight_daemon {
                        c.kill().expect("couldn't kill autobacklight");
                    }
                }
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

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::Event)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
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
