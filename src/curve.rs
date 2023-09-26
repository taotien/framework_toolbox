use anyhow::Result;
use splines::{Interpolation, Key, Spline};

use std::{collections::VecDeque, fs::read_to_string};

const SAMPLE_SIZE: u32 = 100;
// const SAMPLE_INTERVAL_MS: u64 = 100;
// const FPS: u32 = 60;
// const TPF: u64 = 1000 / FPS as u64;
// const HISTERESIS: u32 = 5000;

pub struct Sensor<'a, T> {
    path: &'a str,
    samples: VecDeque<T>,
}

impl Sensor<'_, u32> {
    pub fn new(path: &'static str, initial: u32) -> Result<Self> {
        Ok(Self {
            path,
            samples: VecDeque::from([initial; SAMPLE_SIZE as usize]),
        })
    }

    pub fn sample(&mut self) -> Result<()> {
        self.samples.pop_front();
        self.samples.push_back(Self::get(self)?);
        Ok(())
    }

    pub fn average(&self) -> u32 {
        self.samples.iter().sum::<u32>() / SAMPLE_SIZE
    }

    fn get(&self) -> Result<u32> {
        Ok(read_to_string(self.path)?.trim().parse()?)
    }
}

pub trait Monotonic<T, U> {
    fn add(&mut self, k: T, v: U);
}

impl Monotonic<f32, f32> for Spline<f32, f32> {
    fn add(&mut self, k: f32, v: f32) {
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
