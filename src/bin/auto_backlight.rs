use self::Brightnessctl::*;
use anyhow::Result;
use std::fs::read_to_string;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<()> {
    let mut conf = Config {
        averaging: 5,
        sample_ms: 100,
        offset: 42069,
        fps: 60,
        transition_ms: 1000,
        histeresis: 1000,
    };
    let max = brightnessctl(GetMax)?;
    let scale = max / 3355;
    let smooth = conf.transition_ms / conf.fps;
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
                conf.offset += changed;
                current_prev = current;
                sleep(Duration::from_millis(conf.sample_ms));
            }
            sleep(Duration::from_millis(conf.sample_ms));
            if idx >= conf.averaging - 1 {
                let target = ambient * scale + conf.offset;
                let adjust = target - current;
                if adjust.abs() > conf.histeresis {
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
}

struct Config {
    averaging: usize,
    sample_ms: u64,
    offset: i32,
    fps: u64,
    transition_ms: u64,
    histeresis: i32,
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
            b.arg(format!("{}-", v.abs().to_string()));
        }
        Brightnessctl::Adjust(v) => {
            b.arg("set");
            b.arg(format!("+{}", v.to_string()));
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
