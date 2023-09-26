#![allow(dead_code, unused_imports)]

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

use framework_toolbox::curve::{Monotonic, Sensor};

const SAMPLE_SIZE: u32 = 100;
const SAMPLE_INTERVAL_MS: u64 = 100;
const FPS: u32 = 60;
const TPF: u64 = 1000 / FPS as u64;
const SENSOR_MAX: u32 = 3355;
// const HISTERESIS: u32 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(tokio_unstable)]
    console_subscriber::init();

    let mut backlight = Backlight::new().await?;
    let mut sensor = Sensor::new(
        "/sys/bus/iio/devices/iio:device0/in_illuminance_raw",
        SENSOR_MAX / 2,
    )?;
    let average = Arc::new(AtomicU32::new(SENSOR_MAX / 2));

    let avg = average.clone();
    let sample: JoinHandle<Result<()>> = spawn(async move {
        loop {
            sensor.sample()?;
            avg.store(sensor.average(), Ordering::Relaxed);
            sleep(Duration::from_millis(SAMPLE_INTERVAL_MS)).await;
        }
    });

    let avg = average.clone();
    let adjust_retain: JoinHandle<Result<()>> = spawn(async move {
        loop {
            if !backlight.changed().await? {
                let duration;
                if Backlight::get().await? != backlight.target {
                    backlight.adjust().await?;
                    duration = Duration::from_millis(TPF);
                } else {
                    duration = Duration::from_millis(SAMPLE_INTERVAL_MS * 10);
                }
                backlight.prepare(avg.load(Ordering::Relaxed)).await?;
                sleep(duration).await;
            } else {
                backlight.retain(avg.load(Ordering::Relaxed)).await?;
            }
        }
    });

    let _ = join![sample, adjust_retain];
    unreachable!()
}

struct Backlight {
    requested: u32,
    target: u32,
    diff: i32,
    step: i32,
    curve: Spline<f32, f32>,
}

impl Backlight {
    async fn prepare(&mut self, s: u32) -> Result<()> {
        self.target = self.curve.clamped_sample(s as f32).unwrap() as u32;
        self.diff = self.target as i32 - Self::get().await? as i32;
        self.step = self.diff / FPS as i32;
        Ok(())
    }

    async fn adjust(&mut self) -> Result<()> {
        self.diff = self.target as i32 - Self::get().await? as i32;
        if self.step == 0 {
            self.step = if self.diff > 0 { 1 } else { -1 }
        }
        let v = Self::get().await? as i32 + self.step;
        if v < 0 {
            return Ok(());
        }
        self.set(v as u32).await?;
        Ok(())
    }

    async fn retain(&mut self, s: u32) -> Result<()> {
        sleep(Duration::from_secs(5)).await;
        let current = Self::get().await?;

        if current == 0 {
            return Ok(());
        }

        self.requested = current;
        self.curve
            .add(Key::new(s as f32, current as f32, Interpolation::default()));
        self.prepare(s).await?;
        Ok(())
    }

    async fn changed(&self) -> Result<bool> {
        Ok(Self::get().await? as i32 - self.requested as i32 != 0)
    }

    async fn get() -> Result<u32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/brightness")
                .await?
                .trim()
                .parse()?,
        )
    }

    async fn set(&mut self, val: u32) -> Result<()> {
        write(
            "/sys/class/backlight/intel_backlight/brightness",
            val.to_string(),
        )
        .await?;
        self.requested = val;
        Ok(())
    }

    async fn max() -> Result<u32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/max_brightness")
                .await?
                .trim()
                .parse()?,
        )
    }
    async fn new() -> Result<Self> {
        let current = Self::get().await?;
        let requested = current;
        let target = current;
        let diff = 0;
        let step = 0;
        let floor = Key::new(0., 1., Interpolation::default());
        let ceil = Key::new(3355., Self::max().await? as f32, Interpolation::default());
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
