use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::time::Duration;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, row, slider, text, toggler,
};
use iced::{
    alignment, executor, Alignment, Application, Color, Length, Settings, Subscription, Theme,
};

use iced_native::{window, Event};

use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize)]
struct Toolbox {
    battery_limit: u8,
    fan_duty: u8,
    fan_auto: bool,
    backlight_auto: bool,
    #[serde(skip)]
    backlight_daemon: Option<Child>,
    #[serde(skip)]
    daemon: Option<ChildStdin>,
    #[serde(skip)]
    should_exit: bool,
}

#[derive(Clone, Debug)]
pub enum Message {
    Event(Event),
    BatteryLimitChanged(u8),
    FanDutyChanged(u8),
    FanAutoToggled(bool),
    BacklightAutoToggled(bool),
    Update,
    // Apply,
    Save,
}

impl Application for Toolbox {
    type Message = Message;
    type Executor = executor::Default;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Toolbox, iced::Command<Message>) {
        // elevate daemon at start rather than wait for user interaction
        let mut daemon = Command::new("pkexec")
            .arg("fwtbd")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .spawn()
            .expect("failed to open daemon");
        // hold onto the stdin to communicate and keep process alive
        let daemon_stdin = daemon.stdin.take().expect("couldn't take stdin of daemon");

        // check for existing config, otherwise default
        let mut tb: Toolbox;
        let mut conf = dirs::config_dir().unwrap();
        conf.push("fwtb.toml");
        match read_to_string(conf) {
            Ok(s) => {
                tb = toml_edit::easy::from_str(&s).unwrap();
                tb.daemon = Some(daemon_stdin);
            }
            Err(_) => {
                tb = Toolbox {
                    battery_limit: 69,
                    fan_duty: 42,
                    fan_auto: true,
                    backlight_auto: true,
                    backlight_daemon: None,
                    daemon: Some(daemon_stdin),
                    should_exit: false,
                };
            }
        }

        if tb.backlight_auto {
            tb.backlight_daemon = Some(
                Command::new("fwtb-ab")
                    .spawn()
                    .expect("couldn't start autobacklight"),
            )
        }

        (tb, iced::Command::none())
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::BatteryLimitChanged(value) => {
                self.battery_limit = value;
                writeln!(
                    self.daemon.as_ref().unwrap(),
                    "charge\n{}",
                    self.battery_limit
                )
                .expect("couldn't write to daemon");
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
                writeln!(self.daemon.as_ref().unwrap(), "fan\n{}", self.fan_duty)
                    .expect("couldn't write to daemon");
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
                if !value {
                    writeln!(self.daemon.as_ref().unwrap(), "fan\n{}", self.fan_duty)
                        .expect("couldn't write to daemon");
                } else {
                    writeln!(self.daemon.as_ref().unwrap(), "autofan")
                        .expect("couldn't write to daemon");
                }
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
            Message::Update => {
                writeln!(
                    self.daemon.as_ref().unwrap(),
                    "charge\n{}",
                    self.battery_limit
                )
                .expect("couldn't write to daemon");
            }
            Message::Save => {
                let toml = toml_edit::easy::to_string(&self).unwrap();
                let mut conf = dirs::config_dir().unwrap();
                conf.push("fwtb.toml");
                let mut f = File::create(conf).unwrap();
                f.write_all(toml.as_bytes()).unwrap();
            }
            Message::Event(event) => {
                // TODO
                // fwtbd kills itself when stdin is dropped
                // perhaps autobacklight should do the same so there's no resource leaks
                // or find other non-hacky workaround
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
        let backlight_auto_toggler = toggler(
            String::from("Auto"),
            self.backlight_auto,
            Message::BacklightAutoToggled,
        )
        .text_alignment(alignment::Horizontal::Right)
        .width(Length::Shrink)
        .spacing(5);

        let backlight_controls = column![backlight_auto_toggler].align_items(Alignment::End);

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
                    "Off".to_string()
                }
            })),
            backlight_controls,
            // button("Apply").on_press(Message::Apply),
            button("Save").on_press(Message::Save),
        ]
        .spacing(10)
        .padding(10)
        .align_items(Alignment::Center);

        container(content).center_x().into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let subs = vec![
            iced_native::subscription::events().map(Message::Event),
            iced::time::every(Duration::from_secs(5)).map(|_| Message::Update), // dunno why a closure is needed here
        ];
        iced_native::Subscription::batch(subs)
    }

    fn title(&self) -> String {
        String::from("Framework Toolbox")
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }
}
