use std::fs::read_to_string;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use strum::AsRefStr;

mod gui;

#[derive(Deserialize, Serialize)]
pub struct Toolbox {
    battery_limit: u8,
    fan_duty: u8,
    fan_auto: bool,
    // backlight_auto: bool,
    led_power: Option<LedColor>,
    led_left: Option<LedColor>,
    led_right: Option<LedColor>,
}

impl Default for Toolbox {
    fn default() -> Self {
        Toolbox {
            battery_limit: 87,
            fan_duty: 69,
            fan_auto: true,
            // backlight_auto: true,
            led_power: Some(LedColor::default()),
            led_left: Some(LedColor::default()),
            led_right: Some(LedColor::default()),
        }
    }
}

impl Toolbox {
    pub fn parse() -> Result<Self> {
        let mut path = dirs::config_dir().context("can't find config dir")?;
        path.push("fwtb.toml");
        let file = read_to_string(path).context("can't find fwtb.toml")?;
        let conf = toml::from_str(&file).context("can't parse fwtb.toml")?;
        Ok(conf)
    }
}

#[derive(Default, Deserialize, Serialize, AsRefStr, Debug, Clone, PartialEq, Copy)]
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
