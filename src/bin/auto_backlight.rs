use anyhow::Result;
use splines::{Interpolation, Key, Spline};

use std::{
    collections::VecDeque,
    fs::{read_to_string, write},
    thread::sleep,
    time::Duration,
};

const SAMPLE_SIZE: usize = 100;
const SAMPLE_INTERVAL_MS: u64 = 100;
const BPS: u64 = 60;
const HISTERESIS: i32 = 5000;

// TODO histeresis based on sensor rather than after math
// TODO increase histeresis if ambient is fluctuating a lot
// TODO perhaps cache calculated curve results
// TODO use mki to hook global hotkeys so we don't have to do janky detection
fn main() -> Result<()> {
    let mut b = Brightness::new();
    let mut samples = VecDeque::from([sensor()?; SAMPLE_SIZE]);

    let floor = Key::new(0., 1., Interpolation::default());
    let ceil = Key::new(3355., b.max.into(), Interpolation::default());
    let mut curve = Spline::from_vec(vec![floor, ceil]);

    let sample_interval = Duration::from_millis(SAMPLE_INTERVAL_MS);
    let adjust_interval = Duration::from_millis(1000 / BPS);
    let mut interval = sample_interval;
    let mut step = 0;
    let mut stepper = (0..0).fuse().peekable();
    loop {
        sleep(interval);

        if !b.changed()? {
            // brightness wasn't changed externally, keep sampling and transitioning
            samples.pop_front();
            samples.push_back(sensor()?);
            let avg = samples.iter().sum::<i32>() / samples.len() as i32;
            let target = curve.clamped_sample(avg.into()).unwrap() as i32;
            let diff = target - Brightness::get()?;
            match stepper.next() {
                None => {
                    if diff != 0 && diff.abs() > HISTERESIS {
                        step = diff / BPS as i32;
                        stepper = (0..BPS).fuse().peekable();
                        interval = adjust_interval;
                    }
                }
                Some(_) => {
                    if step == 0 {
                        step = if diff > 0 { 1 } else { -1 };
                    }
                    let adjust = Brightness::get()? + step;
                    b.set(adjust)?;
                    if stepper.peek().is_none() || Brightness::get()? == target {
                        stepper = (0..0).fuse().peekable();
                        interval = sample_interval;
                    }
                }
            }
        } else {
            // brightness was adjusted externally

            // TODO
            // check if change was due to idle
            // on KDE it's 50% of set, then 25% of that
            // maybe libinput can help with this?
            // if c == 23456 {
            //     todo!()
            // } else {

            // wait for user to finish adjusting
            sleep(Duration::from_secs(5));
            interval = sample_interval;

            let avg = samples.iter().sum::<i32>() / samples.len() as i32;
            let current = Brightness::get()?;
            if current == 0 {
                // display set to black (likely from sleep), do nothing
                continue;
            }
            b.requested = current;

            // don't resume previous adjustment
            stepper = (0..0).fuse().peekable();

            curve.monotonic_add(avg.into(), current.into());
        }
    }
}

trait Monotonic<T, U> {
    fn monotonic_add(&mut self, k: T, v: U);
}

impl Monotonic<f64, f64> for Spline<f64, f64> {
    fn monotonic_add(&mut self, k: f64, v: f64) {
        // check if key exists and update or add new key
        if let Some(key) = self.keys().iter().position(|&key| key.t == k) {
            *self.get_mut(key).unwrap().value = v;
        } else {
            let k = Key::new(k, v, Interpolation::default());
            self.add(k);
        }

        // make keys with values in the wrong direction consistent
        if let Some(idx) = self.keys().iter().position(|&key| {
            (key.t != 0. && key.t != 3355.)
                && ((key.value > v && key.t < k) || (key.value < v && key.t > k))
        }) {
            *self.get_mut(idx).unwrap().value = v;
        }
    }
}

fn sensor() -> Result<i32> {
    Ok(
        read_to_string("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")?
            .trim()
            .parse()?,
    )
}

struct Brightness {
    requested: i32,
    max: i32,
}

impl Brightness {
    fn changed(&self) -> Result<bool> {
        let diff = Self::get()? - self.requested;
        Ok(diff != 0)
    }

    fn get() -> Result<i32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/brightness")?
                .trim()
                .parse()?,
        )
    }

    fn set(&mut self, val: i32) -> Result<()> {
        write(
            "/sys/class/backlight/intel_backlight/brightness",
            val.to_string(),
        )?;
        self.requested = val;
        Ok(())
    }

    fn get_max() -> Result<i32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/max_brightness")?
                .trim()
                .parse()?,
        )
    }

    fn new() -> Self {
        Self {
            requested: Self::get().unwrap(),
            max: Self::get_max().unwrap(),
        }
    }
}
