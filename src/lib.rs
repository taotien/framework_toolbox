use anyhow::Result;
use clap::Parser;
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

use std::{fs::read_to_string, process::Child};

pub mod curve;
mod gui;

#[derive(Deserialize, Serialize)]
pub struct Toolbox {
    pub ectool_path: Option<String>,
    battery_limit: u8,
    battery_limit_once: Option<u8>,
    fan_duty: u8,
    fan_auto: bool,
    backlight_auto: bool,
    led_power: Option<LedColor>,
    led_left: Option<LedColor>,
    led_right: Option<LedColor>,

    #[serde(skip)]
    backlight_daemon: Option<Child>,
    #[serde(skip)]
    should_exit: bool,
}

pub struct ToolboxDiff {
    pub battery_limit: Option<u8>,
    pub battery_limit_once: Option<u8>,
    pub fan_duty: Option<u8>,
    pub fan_auto: Option<bool>,
    pub led_power: Option<Option<LedColor>>,
    pub led_left: Option<Option<LedColor>>,
    pub led_right: Option<Option<LedColor>>,
}

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[derive(Default)]
pub struct ToolboxFlags {
    #[arg(short, long)]
    debug: bool,
}

impl Default for Toolbox {
    fn default() -> Self {
        Toolbox {
            ectool_path: None,
            battery_limit: 69,
            battery_limit_once: None,
            fan_duty: 42,
            fan_auto: true,
            backlight_auto: true,
            led_power: Some(LedColor::default()),
            led_left: Some(LedColor::default()),
            led_right: Some(LedColor::default()),
            backlight_daemon: None,
            should_exit: false,
        }
    }
}

impl Toolbox {
    pub fn parse() -> Result<Self> {
        let mut path = dirs::config_dir().unwrap();
        path.push("fwtb.toml");
        let conf = read_to_string(path)?;
        Ok(toml_edit::easy::from_str(&conf)?)
    }

    pub fn diff(&self, new: &Toolbox) -> ToolboxDiff {
        ToolboxDiff {
            battery_limit: {
                if self.battery_limit != new.battery_limit {
                    Some(new.battery_limit)
                } else {
                    None
                }
            },
            battery_limit_once: { new.battery_limit_once },
            fan_duty: {
                if self.fan_duty != new.fan_duty {
                    Some(new.fan_duty)
                } else {
                    None
                }
            },
            fan_auto: {
                if self.fan_auto != new.fan_auto {
                    Some(new.fan_auto)
                } else {
                    None
                }
            },
            led_power: {
                if self.led_power != new.led_power {
                    Some(new.led_power)
                } else {
                    None
                }
            },
            led_left: {
                if self.led_left != new.led_left {
                    Some(new.led_left)
                } else {
                    None
                }
            },
            led_right: {
                if self.led_right != new.led_right {
                    Some(new.led_right)
                } else {
                    None
                }
            },
        }
    }
}

#[derive(Default, Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, AsRefStr)]
pub enum LedColor {
    #[default]
    Auto,
    White,
    Red,
    Green,
    Blue,
    Yellow,
    Amber,
    Off,
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
