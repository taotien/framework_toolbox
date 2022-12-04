use anyhow::Result;
use splines::{Interpolation, Key, Spline};

use std::collections::VecDeque;
use std::fs::{read_to_string, File};
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

const SAMPLING: u64 = 100;
const HISTERESIS: i32 = 5000; // causes large jumps to hitch at the start, but may save cpu?

// impl Spline<f64, f64> {}

// TODO perhaps cache calculated curve results
fn main() -> Result<()> {
    let mut b = Brightness::default();
    let mut running = VecDeque::from([sensor()?; 10]);
    let max = max()?;

    let floor = Key::new(0., 100., Interpolation::Linear);
    let ceil = Key::new(3355., max.into(), Interpolation::default());
    let mut curve = Spline::from_vec(vec![floor, ceil]);

    // let keys = (0..10).map(|i| i * 335); //.collect();
    // let mut vals: Vec<i32> = (0..10).map(|i| i * max / 10).collect();
    // vals[0] = 100;
    // let mut curve: Spline<f64, f64> = Spline::from_vec(vec![]);
    // for (i, k) in keys.clone().enumerate() {
    //     let k = Key::new(k.into(), vals[i].into(), Interpolation::Linear);
    //     curve.add(k);
    // }

    let mut interval = Duration::from_millis(SAMPLING);
    let mut step = 0;
    let mut target: i32;
    let mut stepper = (0..0).fuse();

    loop {
        sleep(interval);
        match b.changed() {
            None => {
                running.pop_front();
                running.push_back(sensor()?);
                let avg: i32 = running.iter().sum::<i32>() / running.len() as i32;
                target = curve.clamped_sample(avg.into()).unwrap() as i32;
                let diff = target - b.get();
                println!("{target}, {diff}");
                match stepper.next() {
                    None => {
                        if diff != 0 && diff.abs() > HISTERESIS {
                            step = diff / 60;
                            stepper = (0..60).fuse();
                            interval = Duration::from_millis(16);
                        }
                    }
                    Some(i) => {
                        // TODO this can be cleaned up if using histeresis?
                        if step == 0 {
                            step = if diff > 0 { 1 } else { -1 };
                        }
                        let adj = b.get() + step;
                        b.set(adj)?;
                        if i == 59 || b.get() == target {
                            stepper = (0..0).fuse();
                            interval = Duration::from_millis(SAMPLING);
                        }
                    }
                }
            }
            Some(c) => {
                // check if change was due to idle
                // maybe libinput can help with this?
                if c == 23456 {
                    todo!()
                } else {
                    sleep(Duration::from_secs(5));
                    let avg: i32 = running.iter().sum::<i32>() / running.len() as i32;
                    let current = b.get();
                    if current < 100 {
                        // device went to sleep, don't do anything
                        // figure out what idel reduction is
                        continue;
                    }
                    b.as_set = current;

                    curve_add(&mut curve, avg.into(), current.into());

                    // find if current ambient already exists
                    // if let Some(k) = curve.keys().iter().position(|&k| k.t == avg.into()) {
                    //     *curve.get_mut(k).unwrap().value = current.into();
                    // } else {
                    //     let k = Key::new(avg.into(), current.into(), Interpolation::Linear);
                    //     curve.add(k);
                    // }

                    // if let Some(i) = keys.clone().position(|i| (avg - i).abs() <= 167) {
                    //     let current = b.get();
                    //     *curve.get_mut(i).unwrap().value = current.into();
                    //     vals[i] = current;
                    // };
                    // println!("{keys:?}\n{vals:?}");
                }
            }
        }
    }
}

fn curve_add(curve: &mut Spline<f64, f64>, k: f64, v: f64) {
    // TODO check if k is idx 0 or len-1

    // checks if key already exists and updates it
    if let Some(key) = curve.keys().iter().position(|&key| key.t == k) {
        *curve.get_mut(key).unwrap().value = v;
    } else {
        let k = Key::new(k, v, Interpolation::Linear);
        curve.add(k);
    }

    // check before
    if let Some(idx) = curve.keys().iter().position(|&key| {
        (key.t != 0. && key.t != 3355.)
            && ((key.value > v && key.t < k) || (key.value < v && key.t > k))
    }) {
        curve.remove(idx);
    }

    for k in curve.keys().iter() {
        for _ in 0..(k.value / 100.) as i32 {
            print!(".");
        }
        println!()
    }
}

struct Brightness {
    as_set: i32,
    current: i32,
}

impl Default for Brightness {
    fn default() -> Self {
        let current = read().unwrap();
        Brightness {
            as_set: current,
            current,
        }
    }
}

impl Brightness {
    fn get(&mut self) -> i32 {
        self.current = read().unwrap();
        self.current
    }

    fn set(&mut self, val: i32) -> Result<()> {
        write(val)?;
        self.as_set = val;
        self.current = val;
        Ok(())
    }

    // fn set_smooth(&mut self, val: i32) -> Result<()> {}

    // Returns Some of the new value, or None if user hasn't changed it since last set by us
    fn changed(&mut self) -> Option<i32> {
        let diff = self.get() - self.as_set;
        if diff != 0 {
            let diff = self.get() - self.as_set;
            Some(diff)
        } else {
            None
        }
    }
}

fn read() -> Result<i32> {
    Ok(
        read_to_string("/sys/class/backlight/intel_backlight/brightness")?
            .trim()
            .parse()?,
    )
}

fn max() -> Result<i32> {
    Ok(
        read_to_string("/sys/class/backlight/intel_backlight/max_brightness")?
            .trim()
            .parse()?,
    )
}

fn write(val: i32) -> Result<()> {
    let mut f = File::create("/sys/class/backlight/intel_backlight/brightness")?;
    f.write_all(&val.to_string().into_bytes())?;
    Ok(())
}

fn sensor() -> Result<i32> {
    Ok(
        read_to_string("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")?
            .trim()
            .parse()?,
    )
}
