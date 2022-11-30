use anyhow::Result;
use splines::{Interpolation, Key, Spline};

use self::Brightnessctl::*;

use std::fs::read_to_string;
use std::thread::sleep;
use std::time::Duration;

// TODO this can probably be merged back into main rather than a separate binary, or completely separted into a crate
// TODO re-introduce histeris option?
// TODO time-of-day/location-based brightnesses, as sensor isn't perfect?
// TODO do this stuff asyncronously
fn main() -> Result<()> {
    // TODO read this from file
    let conf = Config {
        averaging: 5,
        sample_ms: 100,
        fps: 60,
        transition_ms: 1000,
    };
    let max = brightnessctl(GetMax)?;
    let smooth = conf.transition_ms / conf.fps;

    // should've thought of this earlier
    let start = Key::new(0., 100., Interpolation::Linear);
    let end = Key::new(3355., max.into(), Interpolation::default());
    let mut curve = Spline::from_vec(vec![start, end]);

    let mut avg = Vec::with_capacity(conf.averaging);
    for _ in 0..conf.averaging {
        let s = sensor()?;
        avg.push(s);
        sleep(Duration::from_millis(conf.sample_ms));
    }

    let mut current_prev = brightnessctl(Get)?;
    loop {
        for idx in 0..conf.averaging {
            avg[idx] = sensor()?;
            let ambient: i32 = avg.iter().sum::<i32>() / i32::try_from(avg.len()).unwrap();
            let current = brightnessctl(Get)?;
            let changed = current - current_prev;
            if changed != 0 {
                // made long so user settles on value
                sleep(Duration::from_secs(5));
                // TODO clear upper/lower vals so curve is always maintaining direction and not wobbly
                let current = brightnessctl(Get)?;
                let key = curve.keys().iter().position(|&k| k.t == ambient.into());
                match key {
                    Some(k) => {
                        *curve.get_mut(k).unwrap().value = current as f64;
                    }
                    None => {
                        curve.add(Key::new(
                            ambient.into(),
                            current.into(),
                            Interpolation::default(),
                        ));
                    }
                }
            }
            sleep(Duration::from_millis(conf.sample_ms));
            if idx >= conf.averaging - 1 {
                // TODO don't adjust if not much has changed to save battery
                let target: i32 = curve.clamped_sample(ambient.into()).unwrap() as i32;
                let adjust = target - current;
                let step = adjust / smooth as i32;
                for _ in 0..smooth {
                    brightnessctl(Adjust(step))?;
                    sleep(Duration::from_millis(conf.transition_ms / smooth));
                }
                current_prev = brightnessctl(Get)?;
            }
        }
    }
}

struct Config {
    averaging: usize,
    sample_ms: u64,
    fps: u64,
    transition_ms: u64,
}

fn sensor() -> Result<i32> {
    let a = read_to_string("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")?
        .trim()
        .parse()?;
    Ok(a)
}

pub enum Brightnessctl {
    Get,
    Set(i32),
    Adjust(i32),
    GetMax,
}

pub fn brightnessctl(op: Brightnessctl) -> Result<i32> {
    use std::process::Command;
    let mut b = Command::new("brightnessctl");
    match op {
        Brightnessctl::Get => {
            b.arg("get");
        }
        Brightnessctl::Set(v) => {
            b.arg("set");
            b.arg(v.to_string());
        }
        Brightnessctl::Adjust(v) if v.is_negative() => {
            b.arg("set");
            b.arg(format!("{}-", v.abs()));
        }
        Brightnessctl::Adjust(v) => {
            b.arg("set");
            b.arg(format!("+{}", v));
        }
        Brightnessctl::GetMax => {
            b.arg("max");
        }
    }

    let b = b.output()?;
    let b = String::from_utf8(b.stdout)?.trim().parse();
    match b {
        Ok(b) => Ok(b),
        Err(_) => Ok(0),
    }
}
