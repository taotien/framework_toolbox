use std::fs::{read_to_string, File};
use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, slider, text,
    toggler,
};
use iced::{
    alignment, executor, Alignment, Application, Color, Element, Length, Settings, Subscription,
    Theme,
};

use iced_native::{window, Event};

use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize)]
struct Toolbox {
    battery_limit: u8,
    fan_duty: u8,
    fan_auto: bool,
    backlight_auto: bool,
    led_power: Option<LedColor>,
    led_left: Option<LedColor>,
    led_right: Option<LedColor>,

    #[serde(skip)]
    backlight_daemon: Option<Child>,
    #[serde(skip)]
    daemon: Option<ChildStdin>,
    #[serde(skip)]
    should_exit: bool,
}

impl Default for Toolbox {
    fn default() -> Self {
        Toolbox {
            battery_limit: 69,
            fan_duty: 42,
            fan_auto: true,
            backlight_auto: true,
            led_power: Some(LedColor::default()),
            led_left: Some(LedColor::default()),
            led_right: Some(LedColor::default()),
            backlight_daemon: None,
            daemon: None,
            should_exit: false,
        }
    }
}

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
    // Apply,
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
        // elevate daemon at start rather than wait for user interaction
        let mut daemon = Command::new("pkexec")
            .arg("fwtbd")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to open daemon");
        // hold onto the stdin to communicate and keep process alive
        let daemon_stdin = daemon.stdin.take().expect("couldn't take stdin of daemon");

        // check for existing config, otherwise default
        let mut tb: Toolbox;
        let mut conf = dirs::config_dir().unwrap();
        conf.push("fwtb.toml");
        let mut from_conf = false;
        match read_to_string(conf) {
            Ok(s) => {
                tb = toml_edit::easy::from_str(&s).unwrap_or_default();
                from_conf = true;
            }
            Err(_) => {
                tb = Toolbox::default();
            }
        }
        tb.daemon = Some(daemon_stdin);

        if from_conf {
            daemon_write(tb.daemon.as_ref(), "fwchargelimit", tb.battery_limit);
            if tb.fan_auto {
                daemon_write(tb.daemon.as_ref(), "autofanctrl", "");
            } else {
                daemon_write(tb.daemon.as_ref(), "fanduty", tb.fan_duty);
            }
            if let Some(value) = tb.led_power {
                daemon_write(tb.daemon.as_ref(), "led power", value);
            }
            if let Some(value) = tb.led_left {
                daemon_write(tb.daemon.as_ref(), "led left", value);
            }
            if let Some(value) = tb.led_right {
                daemon_write(tb.daemon.as_ref(), "led right", value);
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
                daemon_write(self.daemon.as_ref(), "fwchargelimit", value);
            }
            Message::BatteryOneShot => {
                daemon_write(self.daemon.as_ref(), "fwchargelimit", "100 once");
            }
            Message::FanDutyChanged(value) => {
                self.fan_duty = value;
                self.fan_auto = false;
                daemon_write(self.daemon.as_ref(), "fanduty", value);
            }
            Message::FanAutoToggled(value) => {
                self.fan_auto = value;
                if !value {
                    daemon_write(self.daemon.as_ref(), "fanduty", self.fan_duty);
                } else {
                    daemon_write(self.daemon.as_ref(), "autofanctrl", "");
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
            Message::LEDPowerSelected(value) => {
                self.led_power = Some(value);
                daemon_write(self.daemon.as_ref(), "led power", value);
            }
            Message::LEDLeftSelected(value) => {
                self.led_left = Some(value);
                daemon_write(self.daemon.as_ref(), "led left", value);
            }
            Message::LEDRightSelected(value) => {
                self.led_right = Some(value);
                daemon_write(self.daemon.as_ref(), "led right", value);
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
                // TODO
                // fwtbd kills itself when stdin is dropped
                // perhaps autobacklight should do the same so there's no resource leaks
                // or find other non-hacky workaround
                if let Event::Window(window::Event::CloseRequested) = event {
                    if let Some(c) = &mut self.backlight_daemon {
                        c.kill().expect("couldn't kill autobacklight");
                    }
                    if let Some(c) = &mut self.daemon {
                        // TODO temporary hack
                        daemon_write(self.daemon.as_ref(), "exit", "");
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
            button("Save").on_press(Message::Save),
        ]
        .spacing(space)
        .padding(space)
        .align_items(Alignment::Center)
        .into();

        // container(content.explain(Color::BLACK)).center_x().into()
        container(content).center_x().into()
    }
}

fn daemon_write<T>(daemon: Option<&ChildStdin>, target: &str, value: T)
where
    T: std::fmt::Display,
{
    writeln!(daemon.unwrap(), "{} {}", target, value).expect("couldn't write to daemon!");
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum LedColor {
    Auto,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Amber,
    Off,
}

impl Default for LedColor {
    fn default() -> Self {
        LedColor::Auto
    }
}

impl LedColor {
    const ALL: [LedColor; 8] = [
        LedColor::Auto,
        LedColor::White,
        LedColor::Red,
        LedColor::Green,
        LedColor::Blue,
        LedColor::Yellow,
        LedColor::Amber,
        LedColor::Off,
    ];
}

impl std::fmt::Display for LedColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LedColor::Auto => "Auto",
                LedColor::White => "White",
                LedColor::Red => "Red",
                LedColor::Green => "Green",
                LedColor::Blue => "Blue",
                LedColor::Yellow => "Yellow",
                LedColor::Amber => "Amber",
                LedColor::Off => "Off",
            }
        )
    }
}
