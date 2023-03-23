use anyhow::Result;
use splines::{Interpolation, Key, Spline};
use tokio::{
    fs::{read_to_string, write},
    join, spawn,
    task::JoinHandle,
    time::sleep,
};

use std::{
    collections::VecDeque,
    sync::{atomic::AtomicU32, atomic::Ordering, Arc},
    time::Duration,
};

const SAMPLE_SIZE: u32 = 100;
const SAMPLE_INTERVAL_MS: u64 = 100;
const FPS: u32 = 60;
const TPF: u64 = 1000 / FPS as u64;
// const HISTERESIS: u32 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(tokio_unstable)]
    console_subscriber::init();

    let mut backlight = Backlight::new().await.unwrap();
    let mut sensor = Sensor::new().await.unwrap();
    let average = Arc::new(AtomicU32::new(Sensor::get().await.unwrap()));
    // let (sw, mut sr) = watch::channel(sensor.average);

    let avg = average.clone();
    let sample: JoinHandle<Result<()>> = spawn(async move {
        loop {
            sensor.sample().await.unwrap();
            avg.store(
                sensor.samples.iter().sum::<u32>() / SAMPLE_SIZE,
                Ordering::Relaxed,
            );
            sleep(Duration::from_millis(SAMPLE_INTERVAL_MS)).await;
            // println!("sample");
        }
    });

    // let s = Arc::clone(&sensor);
    let avg = average.clone();
    let adjust_retain: JoinHandle<Result<()>> = spawn(async move {
        loop {
            if !backlight.changed().await.unwrap() {
                let d;
                if Backlight::get().await.unwrap() != backlight.target {
                    backlight.adjust().await.unwrap();
                    d = Duration::from_millis(TPF);
                    // println!(
                    //     "{} {} {}",
                    //     Backlight::get().unwrap(),
                    //     backlight.diff, backlight.target
                    // );
                } else {
                    d = Duration::from_millis(SAMPLE_INTERVAL_MS * 10);
                }
                backlight
                    .prepare(avg.load(Ordering::Relaxed))
                    .await
                    .unwrap();
                sleep(d).await;
                // println!("prepare");
            } else {
                backlight.retain(avg.load(Ordering::Relaxed)).await.unwrap();
                // println!("retain");
            }
        }
    });

    let _ = join![sample, adjust_retain];
    unreachable!()
}

struct Sensor {
    samples: VecDeque<u32>,
}

impl Sensor {
    async fn sample(&mut self) -> Result<()> {
        self.samples.pop_front();
        self.samples.push_back(Self::get().await.unwrap());
        Ok(())
    }

    async fn get() -> Result<u32> {
        Ok(
            read_to_string("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")
                .await
                .unwrap()
                .trim()
                .parse()
                .unwrap(),
        )
    }
    async fn new() -> Result<Self> {
        let samples = VecDeque::from([Self::get().await.unwrap(); SAMPLE_SIZE as usize]);
        Ok(Self { samples })
    }
}

struct Backlight {
    requested: u32,
    target: u32,
    diff: i32,
    step: i32,
    curve: Spline<f32, f32>,
    // curve_map:
}

impl Backlight {
    async fn prepare(&mut self, s: u32) -> Result<()> {
        self.target = self.curve.clamped_sample(s as f32).unwrap() as u32;
        self.diff = self.target as i32 - Self::get().await.unwrap() as i32;
        self.step = self.diff / FPS as i32;
        Ok(())
    }

    async fn adjust(&mut self) -> Result<()> {
        self.diff = self.target as i32 - Self::get().await.unwrap() as i32;
        if self.step == 0 {
            self.step = if self.diff > 0 { 1 } else { -1 }
        }
        let v = Self::get().await.unwrap() as i32 + self.step;
        if v < 0 {
            return Ok(());
        }
        println!("{} {}", self.step, v);
        self.set(v as u32).await.unwrap();
        Ok(())
    }

    async fn retain(&mut self, s: u32) -> Result<()> {
        sleep(Duration::from_secs(5)).await;
        let current = Self::get().await.unwrap();

        if current == 0 {
            return Ok(());
        }

        self.requested = current;
        self.curve.monotonic_add(s as f32, current as f32);
        self.prepare(s).await.unwrap();
        Ok(())
    }

    async fn changed(&self) -> Result<bool> {
        Ok(Self::get().await.unwrap() as i32 - self.requested as i32 != 0)
    }

    async fn get() -> Result<u32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/brightness")
                .await
                .unwrap()
                .trim()
                .parse()
                .unwrap(),
        )
    }

    async fn set(&mut self, val: u32) -> Result<()> {
        // println!("{}", val);
        write(
            "/sys/class/backlight/intel_backlight/brightness",
            val.to_string(),
        )
        .await
        .unwrap();
        self.requested = val;
        Ok(())
    }

    async fn max() -> Result<u32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/max_brightness")
                .await
                .unwrap()
                .trim()
                .parse()
                .unwrap(),
        )
    }
    async fn new() -> Result<Self> {
        let current = Self::get().await.unwrap();
        let requested = current;
        let target = current;
        let diff = 0;
        let step = 0;
        let floor = Key::new(0., 1., Interpolation::default());
        let ceil = Key::new(
            3355.,
            Self::max().await.unwrap() as f32,
            Interpolation::default(),
        );
        let curve = Spline::from_vec(vec![floor, ceil]);
        Ok(Self {
            requested,
            target,
            diff,
            step,
            curve,
        })
    }
}

trait Monotonic<T, U> {
    fn monotonic_add(&mut self, k: T, v: U);
}

impl Monotonic<f32, f32> for Spline<f32, f32> {
    fn monotonic_add(&mut self, k: f32, v: f32) {
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
