use anyhow::Result;
use splines::{Interpolation, Key, Spline};
use tokio::{
    fs::{read_to_string, write},
    join, spawn,
    sync::Mutex,
    task::JoinHandle,
    time::sleep,
};

use std::{collections::VecDeque, sync::Arc, time::Duration};

const SAMPLE_SIZE: usize = 10;
const SAMPLE_INTERVAL_MS: u64 = 1000;
const FPS: u32 = 60;
const HISTERESIS: i32 = 5000;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(tokio_unstable)]
    console_subscriber::init();

    let b = Arc::new(Mutex::new(Backlight::new().await?));

    let a = Arc::clone(&b);
    let adjust: JoinHandle<Result<()>> = spawn(async move {
        loop {
            sleep(Duration::from_millis(1000 / FPS as u64)).await;
            let mut a = a.lock().await;
            if !a.changed().await? {
                a.adjust().await?;
            }
        }
    });

    let s = Arc::clone(&b);
    let sample: JoinHandle<Result<()>> = spawn(async move {
        loop {
            sleep(Duration::from_millis(SAMPLE_INTERVAL_MS)).await;
            let mut s = s.lock().await;
            s.sample().await?;
        }
    });

    let r = Arc::clone(&b);
    let retain: JoinHandle<Result<()>> = spawn(async move {
        loop {
            sleep(Duration::from_millis(SAMPLE_INTERVAL_MS)).await;
            let mut r = r.lock().await;
            if r.changed().await? {
                r.retain().await?;
            }
        }
    });

    let _ = join![retain, sample, adjust];
    unreachable!()
}

#[derive(Debug)]
struct Backlight {
    requested: u32,
    samples: VecDeque<u32>,
    sample_map: Vec<u32>,
    curve: Spline<f32, f32>,
    diff: i32,
}

impl Backlight {
    async fn adjust(&mut self) -> Result<()> {
        if self.diff != 0 {
            let mut step = self.diff / FPS as i32;
            if step == 0 {
                step = if self.diff > 0 { 1 } else { -1 };
            }
            let v = Self::get().await? as i32 + step;

            self.set(v as u32).await?;
        }
        Ok(())
    }

    async fn sample(&mut self) -> Result<()> {
        self.samples.pop_front();
        self.samples.push_back(Self::sensor().await?);
        let target = self.curve.clamped_sample(self.average() as f32).unwrap();
        // let target = self.sample_map[self.average() as usize];
        self.diff = target as i32 - Self::get().await? as i32;

        Ok(())
    }

    async fn retain(&mut self) -> Result<()> {
        sleep(Duration::from_secs(5)).await;
        let current = Self::get().await?;

        if current == 0 {
            return Ok(());
        }

        self.requested = current;
        self.curve
            .monotonic_add(self.average() as f32, current as f32);

        // for i in 0..self.curve.len() {
        //     self.sample_map[i] = self.curve.clamped_sample(i as f32).unwrap() as u32;
        // }

        Ok(())
    }

    async fn get() -> Result<u32> {
        Ok(
            read_to_string("/sys/class/backlight/intel_backlight/brightness")
                .await?
                .trim()
                .parse()?,
        )
    }

    async fn sensor() -> Result<u32> {
        Ok(
            read_to_string("/sys/bus/iio/devices/iio:device0/in_illuminance_raw")
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

    fn average(&self) -> u32 {
        self.samples.iter().sum::<u32>() / self.samples.len() as u32
    }

    async fn changed(&self) -> Result<bool> {
        let diff: i32 = Self::get().await? as i32 - self.requested as i32;
        Ok(diff != 0)
    }

    async fn new() -> Result<Self> {
        let requested = Self::get().await?;
        let samples = VecDeque::from([Self::sensor().await?; SAMPLE_SIZE]);
        let floor = Key::new(0., 1., Interpolation::default());
        let ceil = Key::new(3355., Self::max().await? as f32, Interpolation::default());
        let curve = Spline::from_vec(vec![floor, ceil]);
        let mut sample_map = Vec::with_capacity(3355 + 1);
        // for i in 0..curve.len() {
        //     sample_map.push(curve.clamped_sample(i as f32).unwrap() as u32);
        // }
        let diff = 0;
        Ok(Self {
            requested,
            samples,
            sample_map,
            curve,
            diff,
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
